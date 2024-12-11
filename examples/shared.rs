use memoize::memoize;

#[memoize(SharedCache)]
fn hello(arg: String, arg2: usize) -> bool {
    println!("{} => {}", arg, arg2);
    arg.len() % 2 == arg2
}

fn main() {
    // `hello` is only called once here.
    assert!(!hello("World".to_string(), 0));
    assert!(!hello("World".to_string(), 0));
    // Sometimes one might need the original function.
    assert!(!memoized_original_hello("World".to_string(), 0));
    assert_eq!(memoized_size_hello(), 1);
    memoized_flush_hello();
    assert_eq!(memoized_size_hello(), 0);
}
