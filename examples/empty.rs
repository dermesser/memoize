//! Reproduces (verifies) issue #17: Panics when used on fn without args.

use memoize::memoize;

#[memoize]
fn hello() -> bool {
    println!("hello!");
    true
}

fn main() {
    // `hello` is only called once here.
    assert!(hello());
    assert_eq!(memoized_size_hello(), 1);
    assert!(hello());
    assert_eq!(memoized_size_hello(), 1);
    memoized_flush_hello();
    assert_eq!(memoized_size_hello(), 0);
    // and again here.
    assert!(hello());
}
