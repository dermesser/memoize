use memoize::memoize;

#[memoize]
fn fib(n: u64) -> u64 {
    if n < 2 {
        1
    } else {
        fib(n - 1) + fib(n - 2)
    }
}

#[memoize]
fn fac(n: u64) -> u64 {
    if n < 2 {
        1
    } else {
        n * fac(n - 1)
    }
}

fn main() {
    let fibs = (0..21).map(fib).collect::<Vec<u64>>();
    println!("fib([0,...,20]) = {:?}", fibs);

    let facs = (0..21).map(fac).collect::<Vec<u64>>();
    println!("fac([0,...,20]) = {:?}", facs);
}
