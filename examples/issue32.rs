use memoize::memoize;

#[memoize]
fn expensive(mut foo: i32) -> i32 {
    foo += 1;
    foo
}

fn main() {
    expensive(7);
}
