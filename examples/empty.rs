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
    assert!(hello());
    memoized_flush_hello();
    // and again here.
    assert!(hello());
}
