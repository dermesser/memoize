#![crate_type = "proc-macro"]

use syn;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

use proc_macro::TokenStream;
use quote::{self, ToTokens};

/*
 * TODO:
 */

/**
 * memoize is an attribute to create a memoized version of a (simple enough) function.
 *
 * So far, it works on functions with one or more arguments which are `Clone`-able, returning a
 * `Clone`-able value. Several clones happen within the storage and recall layer, with the
 * assumption being that `memoize` is used to cache such expensive functions that very few
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
 * See the `examples` for concrete applications.
 */
#[proc_macro_attribute]
pub fn memoize(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
        Ok((t, n)) => { input_types = t; input_names = n; },
        Err(e) => return e.to_compile_error().into()
    }

    let input_tuple_type = quote::quote! { (#(#input_types),*) };

    match &sig.output {
        syn::ReturnType::Default => return_type = quote::quote! { () },
        syn::ReturnType::Type(_, ty) => return_type = ty.to_token_stream(),
    }

    // Construct storage for the memoized keys and return values.
    let store_ident = syn::Ident::new(&map_name.to_uppercase(), sig.span());
    let store = quote::quote! {
        lazy_static::lazy_static! {
            static ref #store_ident : std::sync::Mutex<std::collections::HashMap<#input_tuple_type, #return_type>> =
                std::sync::Mutex::new(std::collections::HashMap::new());
        }
    };

    // Rename original function.
    let mut renamed_fn = func.clone();
    renamed_fn.sig.ident = syn::Ident::new(&renamed_name, func.sig.span());
    let memoized_id = &renamed_fn.sig.ident;

    // Construct memoizer function, which calls the original function.
    let syntax_names_tuple = quote::quote! { (#(#input_names),*) };
    let syntax_names_tuple_cloned = quote::quote! { (#(#input_names.clone()),*) };
    let memoizer = quote::quote! {
        #sig {
            let mut hm = &mut #store_ident.lock().unwrap();
            if let Some(r) = hm.get(&#syntax_names_tuple_cloned) {
                return r.clone();
            }
            let r = #memoized_id(#(#input_names.clone()),*);
            hm.insert(#syntax_names_tuple, r.clone());
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

fn check_signature(sig: &syn::Signature) -> Result<(Vec<Box<syn::Type>>, Vec<Box<syn::Pat>>), syn::Error> {
    if let syn::FnArg::Receiver(_) = sig.inputs[0] {
        return Err(
            syn::Error::new(
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
                return Err(syn::Error::new(sig.span(), "Cannot memoize arbitrary patterns!"));
            }
        }
    }
    Ok((types, names))
}

#[cfg(test)]
mod tests {}
