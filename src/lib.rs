#![crate_type = "proc-macro"]

use syn;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

use proc_macro::TokenStream;
use quote::{self, ToTokens};

trait MemoizeStore<K, V>
where
    K: std::hash::Hash,
{
    fn get(&mut self, k: &K) -> Option<&V>;
    fn put(&mut self, k: K, v: V);
}

impl<K: std::hash::Hash + Eq + Clone, V> MemoizeStore<K, V> for std::collections::HashMap<K, V> {
    fn get(&mut self, k: &K) -> Option<&V> {
        std::collections::HashMap::<K, V>::get(self, k)
    }
    fn put(&mut self, k: K, v: V) {
        self.insert(k, v);
    }
}

#[cfg(not(feature = "full"))]
mod store {
    use proc_macro::TokenStream;

    /// Returns tokenstreams (for quoting) of the store type and an expression to initialize it.
    pub fn construct_cache(
        _attr: TokenStream,
        key_type: proc_macro2::TokenStream,
        value_type: proc_macro2::TokenStream,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        // This is the unbounded default.
        (
            quote::quote! { std::collections::HashMap<#key_type, #value_type> },
            quote::quote! { std::collections::HashMap::new() },
        )
    }

    /// Returns tokenstreams (for quoting) of method names for inserting/getting (first/second
    /// return tuple value).
    pub fn cache_access_methods(
        _attr: &TokenStream,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        (quote::quote! { insert }, quote::quote! { get })
    }
}

#[cfg(feature = "full")]
mod store {
    use super::MemoizeStore;
    use proc_macro::TokenStream;
    use syn::parse as p;

    impl<K: std::hash::Hash + Eq + Clone, V> MemoizeStore<K, V> for lru::LruCache<K, V> {
        fn get(&mut self, k: &K) -> Option<&V> {
            lru::LruCache::<K, V>::get(self, k)
        }
        fn put(&mut self, k: K, v: V) {
            lru::LruCache::<K, V>::put(self, k, v);
        }
    }

    #[derive(Default, Debug, Clone)]
    struct CacheOptions {
        lru_max_entries: Option<usize>,
    }

    syn::custom_keyword!(Capacity);
    syn::custom_punctuation!(Colon, :);

    impl p::Parse for CacheOptions {
        fn parse(input: p::ParseStream) -> syn::Result<Self> {
            let la = input.lookahead1();
            if la.peek(Capacity) {
                let _: Capacity = input.parse().unwrap();
                let _: Colon = input.parse().unwrap();
                let cap: syn::LitInt = input.parse().unwrap();

                return Ok(CacheOptions {
                    lru_max_entries: Some(cap.base10_parse()?),
                });
            }
            Ok(Default::default())
        }
    }

    /// Returns tokenstreams (for quoting) of the store type and an expression to initialize it.
    pub fn construct_cache(
        attr: &TokenStream,
        key_type: proc_macro2::TokenStream,
        value_type: proc_macro2::TokenStream,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let options: CacheOptions = syn::parse(attr.clone()).unwrap();

        // This is the unbounded default.
        match options.lru_max_entries {
            None => (
                quote::quote! { std::collections::HashMap<#key_type, #value_type> },
                quote::quote! { std::collections::HashMap::new() },
            ),
            Some(cap) => (
                quote::quote! { lru::LruCache<#key_type, #value_type> },
                quote::quote! { lru::LruCache::new(#cap) },
            ),
        }
    }

    /// Returns tokenstreams (for quoting) of method names for inserting/getting (first/second
    /// return tuple value).
    pub fn cache_access_methods(
        attr: &TokenStream,
    ) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
        let options: CacheOptions = syn::parse(attr.clone()).unwrap();

        // This is the unbounded default.
        match options.lru_max_entries {
            None => (quote::quote! { insert }, quote::quote! { get }),
            Some(cap) => (quote::quote! { put }, quote::quote! { get }),
        }
    }
}

/**
 * memoize is an attribute to create a memoized version of a (simple enough) function.
 *
 * So far, it works on functions with one or more arguments which are `Clone`- and `Hash`-able,
 * returning a `Clone`-able value. Several clones happen within the storage and recall layer, with
 * the assumption being that `memoize` is used to cache such expensive functions that very few
 * `clone()`s do not matter.
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
 * The `memoize` attribute can take further parameters in order to use an LRU cache:
 * `#[memoize(Capacity: 1234)]`.
 *
 * See the `examples` for concrete applications.
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
    let input_names: Vec<Box<syn::Pat>>;
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

    // Construct storage for the memoized keys and return values.
    let store_ident = syn::Ident::new(&map_name.to_uppercase(), sig.span());
    let (cache_type, cache_init) = store::construct_cache(&attr, input_tuple_type, return_type);
    let store = quote::quote! {
        lazy_static::lazy_static! {
            static ref #store_ident : std::sync::Mutex<#cache_type> =
                std::sync::Mutex::new(#cache_init);
        }
    };

    // Rename original function.
    let mut renamed_fn = func.clone();
    renamed_fn.sig.ident = syn::Ident::new(&renamed_name, func.sig.span());
    let memoized_id = &renamed_fn.sig.ident;

    // Construct memoizer function, which calls the original function.
    let syntax_names_tuple = quote::quote! { (#(#input_names),*) };
    let syntax_names_tuple_cloned = quote::quote! { (#(#input_names.clone()),*) };
    let (insert_fn, get_fn) = store::cache_access_methods(&attr);
    let memoizer = quote::quote! {
        #sig {
            let mut hm = &mut #store_ident.lock().unwrap();
            if let Some(r) = hm.#get_fn(&#syntax_names_tuple_cloned) {
                return r.clone();
            }
            let r = #memoized_id(#(#input_names.clone()),*);
            hm.#insert_fn(#syntax_names_tuple, r.clone());
            r
        }
    };

    (quote::quote! {
        #store

        #renamed_fn

        #memoizer
    })
    .into()
}

fn check_signature(
    sig: &syn::Signature,
) -> Result<(Vec<Box<syn::Type>>, Vec<Box<syn::Pat>>), syn::Error> {
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

            if let syn::Pat::Ident(_) = &*arg.pat {
                names.push(arg.pat.clone());
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
