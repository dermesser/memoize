#![crate_type = "proc-macro"]

use syn;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use quote::{self, ToTokens};

use std::collections::HashMap;
use std::sync::Mutex;

/*
 * TODO:
 * - Functions with multiple arguments.
 */

/**
 * memoize is an attribute to create a memoized version of a (simple enough) function.
 *
 * So far, it works on functions with one argument which is Clone-able, returning a Clone-able
 * value.
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
 * fn hello(arg: String) -> bool {
 *      arg.len()%2 == 0
 * }
 *
 * // `hello` is only called once.
 * assert!(! hello("World".to_string()));
 * assert!(! hello("World".to_string()));
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

    let fn_name = &func.sig.ident.to_string();
    let renamed_name = format!("memoized_original_{}", fn_name);
    let map_name = format!("memoized_mapping_{}", fn_name);

    let mut type_in = None;
    let mut name_in = None;
    let type_out;

    // Only one argument
    // TODO: cache multiple arguments
    if sig.inputs.len() == 1 {
        if let syn::FnArg::Typed(ref arg) = sig.inputs[0] {
            type_in = Some(arg.ty.clone());
            if let syn::Pat::Ident(_) = &*arg.pat {
                name_in = Some(arg.pat.clone());
            } else {
                return syn::Error::new(
                    sig.span(),
                    "Cannot memoize method (self-receiver) without arguments!",
                )
                .to_compile_error()
                .into();
            }
        } else {
            return TokenStream::from(
                syn::Error::new(
                    sig.span(),
                    "Cannot memoize method (self-receiver) without arguments!",
                )
                .to_compile_error(),
            );
        }
    }
    match &sig.output {
        syn::ReturnType::Default => type_out = quote::quote! { () },
        syn::ReturnType::Type(_, ty) => type_out = ty.to_token_stream(),
    }

    let type_in = type_in.unwrap();
    let name_in = name_in.unwrap();
    let store_ident = syn::Ident::new(&map_name.to_uppercase(), sig.span());
    let store = quote::quote! {
        lazy_static::lazy_static! {
            static ref #store_ident : std::sync::Mutex<std::collections::HashMap<#type_in, #type_out>> =
                std::sync::Mutex::new(std::collections::HashMap::new());
        }
    };

    let mut renamed_fn = func.clone();
    renamed_fn.sig.ident = syn::Ident::new(&renamed_name, func.sig.span());
    let memoized_id = &renamed_fn.sig.ident;

    let memoizer = quote::quote! {
        #sig {
            let mut hm = &mut #store_ident.lock().unwrap();
            if let Some(r) = hm.get(&#name_in) {
                return r.clone();
            }
            let r = #memoized_id(#name_in.clone());
            hm.insert(#name_in, r.clone());
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

lazy_static! {
    static ref STORE: Mutex<HashMap<i32, bool>> = Mutex::new(HashMap::new());
}

#[cfg(test)]
mod tests {}
