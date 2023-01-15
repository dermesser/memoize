//! Reproduces (verifies) issue #17: Panics when used on fn without args.

use memoize::memoize;

#[cfg(feature = "full")]
#[memoize(CustomHasher: std::collections::HashMap)]
fn hello() -> bool {
    println!("hello!");
    true
}

// ! This will panic because CustomHasher and Capacity are being used.
// #[cfg(feature = "full")]
// #[memoize(CustomHasher: std::collections::HashMap, Capacity: 3usize)]
// fn will_panic(a: u32, b: u32) -> u32 {
//     a + b
// }

#[cfg(feature = "full")]
fn main() {
    // `hello` is only called once here.
    assert!(hello());
    assert!(hello());
    memoized_flush_hello();
    // and again here.
    assert!(hello());
}

#[cfg(not(feature = "full"))]
fn main() {
    println!("Use the \"full\" feature to execute this example");
}
