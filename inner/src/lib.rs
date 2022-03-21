#![crate_type = "proc-macro"]
#![allow(unused_imports)] // Spurious complaints about a required trait import.

use syn::{self, parse, parse_macro_input, spanned::Spanned, Expr, ItemFn};

use proc_macro::TokenStream;
use quote::{self, ToTokens};

mod kw {
    syn::custom_keyword!(Capacity);
    syn::custom_keyword!(TimeToLive);
    syn::custom_keyword!(SharedCache);
    syn::custom_punctuation!(Colon, :);
}

#[derive(Default, Clone)]
struct CacheOptions {
    lru_max_entries: Option<usize>,
    time_to_live: Option<Expr>,
    shared_cache: bool,
}

#[derive(Clone)]
enum CacheOption {
    LRUMaxEntries(usize),
    TimeToLive(Expr),
    SharedCache,
}

// To extend option parsing, add functionality here.
impl parse::Parse for CacheOption {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        if la.peek(kw::Capacity) {
            #[cfg(not(feature = "full"))]
            return Err(syn::Error::new(input.span(),
            "memoize error: Capacity specified, but the feature 'full' is not enabled! To fix this, compile with `--features=full`.",
            ));

            input.parse::<kw::Capacity>().unwrap();
            input.parse::<kw::Colon>().unwrap();
            let cap: syn::LitInt = input.parse().unwrap();

            return Ok(CacheOption::LRUMaxEntries(cap.base10_parse()?));
        }
        if la.peek(kw::TimeToLive) {
            #[cfg(not(feature = "full"))]
            return Err(syn::Error::new(input.span(),
            "memoize error: TimeToLive specified, but the feature 'full' is not enabled! To fix this, compile with `--features=full`.",
            ));

            input.parse::<kw::TimeToLive>().unwrap();
            input.parse::<kw::Colon>().unwrap();
            let cap: syn::Expr = input.parse().unwrap();

            return Ok(CacheOption::TimeToLive(cap));
        }
        if la.peek(kw::SharedCache) {
            input.parse::<kw::SharedCache>().unwrap();
            return Ok(CacheOption::SharedCache);
        }
        Err(la.error())
    }
}

impl parse::Parse for CacheOptions {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let f: syn::punctuated::Punctuated<CacheOption, syn::Token![,]> =
            input.parse_terminated(CacheOption::parse)?;
        let mut opts = Self::default();

        for opt in f {
            match opt {
                CacheOption::LRUMaxEntries(cap) => opts.lru_max_entries = Some(cap),
                CacheOption::TimeToLive(sec) => opts.time_to_live = Some(sec),
                CacheOption::SharedCache => opts.shared_cache = true,
            }
        }
        Ok(opts)
    }
}

// This implementation of the storage backend does not depend on any more crates.
#[cfg(not(feature = "full"))]
mod store {
    use crate::CacheOptions;
    use proc_macro::TokenStream;

    /// Returns tokenstreams (for quoting) of the store type and an expression to initialize it.
    pub(crate) fn construct_cache(
        _options: &CacheOptions,
        key_type: proc_macro2::TokenStream,
        value_type: proc_macro2::TokenStream,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        // This is the unbounded default.
        (
            quote::quote! { std::collections::HashMap<#key_type, #value_type> },
            quote::quote! { std::collections::HashMap::new() },
        )
    }

    /// Returns names of methods as TokenStreams to insert and get (respectively) elements from a
    /// store.
    pub(crate) fn cache_access_methods(
        _options: &CacheOptions,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        (quote::quote! { insert }, quote::quote! { get })
    }
}

// This implementation of the storage backend also depends on the `lru` crate.
#[cfg(feature = "full")]
mod store {
    use crate::CacheOptions;
    use proc_macro::TokenStream;

    /// Returns TokenStreams to be used in quote!{} for parametrizing the memoize store variable,
    /// and initializing it.
    ///
    /// First return value: Type of store ("Container<K,V>").
    /// Second return value: Initializer syntax ("Container::<K,V>::new()").
    pub(crate) fn construct_cache(
        options: &CacheOptions,
        key_type: proc_macro2::TokenStream,
        value_type: proc_macro2::TokenStream,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let value_type = match options.time_to_live {
            None => quote::quote! {#value_type},
            Some(_) => quote::quote! {(std::time::Instant, #value_type)},
        };
        // This is the unbounded default.
        match options.lru_max_entries {
            None => (
                quote::quote! { std::collections::HashMap<#key_type, #value_type> },
                quote::quote! { std::collections::HashMap::new() },
            ),
            Some(cap) => (
                quote::quote! { ::memoize::lru::LruCache<#key_type, #value_type> },
                quote::quote! { ::memoize::lru::LruCache::new(#cap) },
            ),
        }
    }

    /// Returns names of methods as TokenStreams to insert and get (respectively) elements from a
    /// store.
    pub(crate) fn cache_access_methods(
        options: &CacheOptions,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        // This is the unbounded default.
        match options.lru_max_entries {
            None => (quote::quote! { insert }, quote::quote! { get }),
            Some(_) => (quote::quote! { put }, quote::quote! { get }),
        }
    }
}

/**
 * memoize is an attribute to create a memoized version of a (simple enough) function.
 *
 * So far, it works on functions with one or more arguments which are `Clone`- and `Hash`-able,
 * returning a `Clone`-able value. Several clones happen within the storage and recall layer, with
 * the assumption being that `memoize` is used to cache such expensive functions that very few
 * `clone()`s do not matter. `memoize` doesn't work on methods (functions with `[&/&mut/]self`
 * receiver).
 *
 * Calls are memoized for the lifetime of a program, using a statically allocated, Mutex-protected
 * HashMap.
 *
 * Memoizing functions is very simple: As long as the above-stated requirements are fulfilled,
 * simply use the `#[memoize::memoize]` attribute:
 *
 * ```
 * use memoize::memoize;
 * #[memoize]
 * fn hello(arg: String, arg2: usize) -> bool {
 *      arg.len()%2 == arg2
 * }
 *
 * // `hello` is only called once.
 * assert!(! hello("World".to_string(), 0));
 * assert!(! hello("World".to_string(), 0));
 * ```
 *
 * If you need to use the un-memoized function, it is always available as `memoized_original_{fn}`,
 * in this case: `memoized_original_hello()`.
 *
 * See the `examples` for concrete applications.
 *
 * *The following descriptions need the `full` feature enabled.*
 *
 * The `memoize` attribute can take further parameters in order to use an LRU cache:
 * `#[memoize(Capacity: 1234)]`. In that case, instead of a `HashMap` we use an `lru::LruCache`
 * with the given capacity.
 * `#[memoize(TimeToLive: Duration::from_secs(2))]`. In that case, cached value will be actual
 * no longer than duration provided and refreshed with next request. If you prefer chrono::Duration,
 * it can be also used: `#[memoize(TimeToLive: chrono::Duration::hours(9).to_std().unwrap()]`
 *
 * This mechanism can, in principle, be extended (in the source code) to any other cache mechanism.
 *
 */
#[proc_macro_attribute]
pub fn memoize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let sig = &func.sig;

    let fn_name = &sig.ident.to_string();
    let renamed_name = format!("memoized_original_{}", fn_name);
    let map_name = format!("memoized_mapping_{}", fn_name);

    // Extracted from the function signature.
    let input_types: Vec<Box<syn::Type>>;
    let input_names: Vec<syn::Ident>;
    let return_type;

    match check_signature(sig) {
        Ok((t, n)) => {
            input_types = t;
            input_names = n;
        }
        Err(e) => return e.to_compile_error().into(),
    }

    let input_tuple_type = quote::quote! { (#(#input_types),*) };

    match &sig.output {
        syn::ReturnType::Default => return_type = quote::quote! { () },
        syn::ReturnType::Type(_, ty) => return_type = ty.to_token_stream(),
    }

    // Parse options from macro attributes
    let options: CacheOptions = syn::parse(attr.clone()).unwrap();

    // Construct storage for the memoized keys and return values.
    let store_ident = syn::Ident::new(&map_name.to_uppercase(), sig.span());
    let (cache_type, cache_init) =
        store::construct_cache(&options, input_tuple_type, return_type.clone());
    let store = if options.shared_cache {
        quote::quote! {
            ::memoize::lazy_static::lazy_static! {
                static ref #store_ident : std::sync::Mutex<#cache_type> =
                    std::sync::Mutex::new(#cache_init);
            }
        }
    } else {
        quote::quote! {
            std::thread_local! {
                static #store_ident : std::cell::RefCell<#cache_type> =
                    std::cell::RefCell::new(#cache_init);
            }
        }
    };

    // Rename original function.
    let mut renamed_fn = func.clone();
    renamed_fn.sig.ident = syn::Ident::new(&renamed_name, func.sig.span());
    let memoized_id = &renamed_fn.sig.ident;

    // Construct memoizer function, which calls the original function.
    let syntax_names_tuple = quote::quote! { (#(#input_names),*) };
    let syntax_names_tuple_cloned = quote::quote! { (#(#input_names.clone()),*) };
    let (insert_fn, get_fn) = store::cache_access_methods(&options);
    let (read_memo, memoize) = match options.time_to_live {
        None => (
            quote::quote!(hm.#get_fn(&#syntax_names_tuple_cloned).cloned()),
            quote::quote!(hm.#insert_fn(#syntax_names_tuple, r.clone());),
        ),
        Some(ttl) => (
            quote::quote! {
                hm.#get_fn(&#syntax_names_tuple_cloned).and_then(|(last_updated, r)|
                    (last_updated.elapsed() < #ttl).then(|| r.clone())
                )
            },
            quote::quote!(hm.#insert_fn(#syntax_names_tuple, (std::time::Instant::now(), r.clone()));),
        ),
    };

    let memoizer = if options.shared_cache {
        quote::quote! {
            {
                let mut hm = #store_ident.lock().unwrap();
                if let Some(r) = #read_memo {
                    return r
                }
            }
            let r = #memoized_id(#(#input_names.clone()),*);

            let mut hm = #store_ident.lock().unwrap();
            #memoize

            r
        }
    } else {
        quote::quote! {
            let r = #store_ident.with(|hm| {
                let mut hm = hm.borrow_mut();
                #read_memo
            });
            if let Some(r) = r {
                return r;
            }

            let r = #memoized_id(#(#input_names.clone()),*);

            #store_ident.with(|hm| {
                let mut hm = hm.borrow_mut();
                #memoize
            });

            r
        }
    };

    (quote::quote! {
        #renamed_fn

        #store

        #[allow(unused_variables)]
        #sig {
            #memoizer
        }
    })
    .into()
}

fn check_signature(
    sig: &syn::Signature,
) -> Result<(Vec<Box<syn::Type>>, Vec<syn::Ident>), syn::Error> {
    if let syn::FnArg::Receiver(_) = sig.inputs[0] {
        return Err(syn::Error::new(
            sig.span(),
            "Cannot memoize method (self-receiver) without arguments!",
        ));
    }

    let mut types = vec![];
    let mut names = vec![];
    for a in &sig.inputs {
        if let syn::FnArg::Typed(ref arg) = a {
            types.push(arg.ty.clone());

            if let syn::Pat::Ident(patident) = &*arg.pat {
                names.push(patident.ident.clone());
            } else {
                return Err(syn::Error::new(
                    sig.span(),
                    "Cannot memoize arbitrary patterns!",
                ));
            }
        }
    }
    Ok((types, names))
}

#[cfg(test)]
mod tests {}
