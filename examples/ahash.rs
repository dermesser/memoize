use ahash::{HashMap, HashMapExt};
use memoize::memoize;

#[cfg(feature = "full")]
#[memoize(CustomHasher: HashMap)]
fn hello() -> bool {
    println!("hello!");
    true
}

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
