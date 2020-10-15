#![crate_type = "proc-macro"]

use syn;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{self, ToTokens};

use std::collections::HashMap;
use std::sync::Mutex;

/*
 * TODO:
 * - Create static map for memoized arguments/results
 * - Create memoized version of function
 * - Rename original function to memoized_original_{fn}
 *
 */

#[proc_macro_attribute]
pub fn memoize(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let sig = &func.sig;

    let original_name = &func.sig.ident;
    let fn_name = &func.sig.ident.to_string();
    let renamed_name = format!("memoized_original_{}", fn_name);
    let map_name = format!("memoized_mapping_{}", fn_name);
    println!("{}", fn_name);

    let mut type_in = None;
    let mut name_in = None;
    let type_out;

    // Only one argument
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
    // TODO: Cache methods too
    match &sig.output {
        syn::ReturnType::Default => type_out = quote::quote! { () },
        syn::ReturnType::Type(_, ty) => type_out = ty.to_token_stream(),
    }

    let type_in = type_in.unwrap();
    let map_ident = syn::Ident::new(&map_name, sig.span());
    let store = quote::quote! {
        lazy_static::lazy_static! {
            static ref #map_ident : std::sync::Mutex<std::collections::HashMap<#type_in, #type_out>> =
                std::sync::Mutex::new(std::collections::HashMap::new());
        }
    };

    let mut renamed_fn = func.clone();
    renamed_fn.sig.ident = syn::Ident::new(&renamed_name, func.sig.span());

    let name_in = name_in.unwrap();
    let memoized_id = &renamed_fn.sig.ident;
    let memoizer = quote::quote! {
        #sig {
            let mut hm = &mut #map_ident.lock().unwrap();
            if let Some(r) = hm.get(&#name_in) {
                return *r;
            }
            let r = #memoized_id(#name_in);
            hm.insert(#name_in, r);
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

// fn memoizer(a: i32) -> bool {
//    let mut hm = &mut STORE.lock().unwrap();
//    if let Some(r) = hm.get(&a) {
//        return *r;
//    }
//    let r = memoized_function(a);
//    hm.insert(a, r);
//    r
// }

#[cfg(test)]
mod tests {
    use super::*;
}
