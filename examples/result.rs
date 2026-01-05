use memoize::memoize;

#[memoize]
fn hello(arg: String, arg2: usize) -> std::io::Result<bool> {
    println!("{} => {}", arg, arg2);
    Ok(arg.len() % 2 == arg2)
}

fn main() {
    // `hello` is only called once here.
    assert!(hello("World2".to_string(), 0).unwrap());
    assert!(hello("World2".to_string(), 0).unwrap());
    // Sometimes one might need the original function.
    assert!(memoized_original_hello("World2".to_string(), 0).unwrap());
    memoized_flush_hello();
}
