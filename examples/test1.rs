use memoize::memoize;
use std::time::{Instant, Duration};
use std::thread;

#[derive(Debug, Clone)]
struct ComplexStruct {
    s: String,
    b: bool,
    i: Instant,
}

#[memoize(Capacity: 123, SecondsToLive: 1)]
fn hello(key: String) -> ComplexStruct {
    println!("hello: {}", key);
    ComplexStruct {
        s: key,
        b: false,
        i: Instant::now(),
    }
}

fn main() {
    println!("result: {:?}", hello("ABC".to_string()));
    println!("result: {:?}", hello("DEF".to_string()));
    println!("result: {:?}", hello("ABC".to_string()));  //Same as first
    thread::sleep(Duration::from_millis(2100));
    println!("result: {:?}", hello("ABC".to_string()));  //Refreshed
    println!("result: {:?}", memoized_original_hello("ABC".to_string()));
}
