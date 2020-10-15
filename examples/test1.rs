use memoize::memoize;

#[memoize]
fn hello(a: i32) -> bool {
    println!("HELLO");
    a%2 == 0
}

fn main() {
    println!("result: {}", hello(32));
    println!("result: {}", hello(32));
}
