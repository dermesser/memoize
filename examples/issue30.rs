
use memoize::memoize;

#[derive(Clone)]
struct Struct {}
#[derive(Clone)]
struct Error {}

#[memoize(SharedCache, Capacity: 1024)]
fn my_function(arg: &'static str) -> Result<Struct, Error> {
    println!("{}", arg);
    Ok(Struct{})
}

fn main() {
    let s = "Hello World";
    my_function(s).ok();
}
