# memoize

[![Docs.rs](https://docs.rs/memoize/badge.svg)](https://docs.rs/memoize)
[![Crates.rs](https://img.shields.io/crates/v/memoize.svg)](https://crates.io/crates/memoize)
[![CI](https://github.com/dermesser/rex/workflows/CI/badge.svg)](https://github.com/dermesser/memoize/actions)

A `#[memoize]` attribute for somewhat simple Rust functions: That is, functions
with one or more `Clone`-able arguments, and a `Clone`-able return type. That's it.

Read the documentation (`cargo doc --open`) for the sparse details, or take a
look at the `examples/`, if you want to know more:

```rust
// From examples/test2.rs

use memoize::memoize;

#[memoize]
fn hello(arg: String, arg2: usize) -> bool {
  arg.len()%2 == arg2
}

fn main() {
  // `hello` is only called once here.
  assert!(! hello("World".to_string(), 0));
  assert!(! hello("World".to_string(), 0));
  // Sometimes one might need the original function.
  assert!(! memoized_original_hello("World".to_string(), 0));
}
```

This is expanded into (with a few simplifications):

```rust
// This is obviously further expanded before compiling.
lazy_static! {
  static ref MEMOIZED_MAPPING_HELLO : Mutex<HashMap<String, bool>>;
}

fn memoized_original_hello(arg: String, arg2: usize) -> bool {
  arg.len() % 2 == arg2
}

fn hello(arg: String, arg2: usize) -> bool {
  let mut hm = &mut MEMOIZED_MAPPING_HELLO.lock().unwrap();
  if let Some(r) = hm.get(&(arg.clone(), arg2.clone())) {
    return r.clone();
  }
  let r = memoized_original_hello(arg.clone(), arg2.clone());
  hm.insert((arg, arg2), r.clone());
  r
}
```

## Further Functionality

You can choose to use an [LRU cache](https://crates.io/crates/lru). In fact, if
you know that a memoized function has an unbounded number of different inputs,
you should do this! In that case, use the attribute like this:

```rust
// From examples/test1.rs
// Compile with --features=full
use memoize::memoize;

#[derive(Debug, Clone)]
struct ComplexStruct {
  // ...
}

#[memoize(Capacity: 123)]
fn hello(key: String) -> ComplexStruct {
  // ...
}
```

Adding more caches and configuration options is relatively simple, and a matter
of parsing attribute parameters. Currently, compiling will fail if you use a
parameter such as `Capacity` without the feature `full` being enabled.

Another parameter is TimeToLive, specifying how long a cached value is allowed
to live:

```rust
#[memoize(Capacity: 123, TimeToLive: Duration::from_secs(2))]
```

`chrono::Duration` is also possible, but would have to first be converted to
`std::time::Duration`

```rust
#[memoize(TimeToLive: chrono::Duration::hours(3).to_std().unwrap())]
```

The cached value will never be older than duration provided and instead
recalculated on the next request.

## Contributions

...are always welcome! This being my first procedural-macros crate, I am
grateful for improvements of functionality and style. Please send a pull
request, and don't be discouraged if it takes a while for me to review it; I'm
sometimes a bit slow to catch up here :)   -- Lewin

