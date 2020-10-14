#![crate_type = "proc-macro"]

use proc_macro::TokenStream;

use lazy_static::lazy_static;

use std::collections::HashMap;
use std::sync::Mutex;

#[proc_macro_attribute]
pub fn memoize(attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input
    item
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
