use memoize::memoize;

#[derive(Debug, Clone)]
struct ComplexStruct {
    s: String,
    b: bool,
    i: i32,
}

#[memoize]
fn hello(key: String) -> ComplexStruct {
    println!("hello: {}", key);
    ComplexStruct {
        s: key,
        b: false,
        i: 332,
    }
}

fn main() {
    println!("result: {:?}", hello("ABC".to_string()));
    println!("result: {:?}", hello("DEF".to_string()));
    println!("result: {:?}", hello("ABC".to_string()));
    println!("result: {:?}", memoized_original_hello("ABC".to_string()));
}
