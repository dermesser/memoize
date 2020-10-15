# memoize

A `#[memoize]` attribute for somewhat simple Rust functions. That's it.

Read the documentation (`cargo doc --open`) for the sparse details, or take a
look at the `examples/`, if you want to know more:

```rust
use memoize::memoize;
#[memoize]
fn hello(arg: String) -> bool {
     arg.len()%2 == 0
}

// `hello` is only called once here.
assert!(! hello("World".to_string()));
assert!(! hello("World".to_string()));
// Sometimes one might need the original function.
assert!(! memoized_original_hello("World".to_string()));
```

Intentionally not yet on crates.rs.
