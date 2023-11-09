use memoize::memoize;

/// Wrapper struct for a [`u32`].
///
/// Note that A deliberately does not implement [`Clone`] or [`Hash`], to demonstrate that it can be
/// passed through.
struct C {
    c: u32
}

#[memoize(Ignore: a, Ignore: c)]
fn add(a: u32, b: u32, c: C, d: u32) -> u32 {
    a + b + c.c + d
}

#[memoize(Ignore: call_count, SharedCache)]
fn add2(a: u32, b: u32, call_count: &mut u32) -> u32 {
    *call_count += 1;
    a + b
}

fn main() {
    // Note that the third argument is not `Clone` but can still be passed through.
    assert_eq!(add(1, 2, C {c: 3}, 4), 10);

    assert_eq!(add(3, 2, C {c: 4}, 4), 10);
    memoized_flush_add();

    // Once cleared, all arguments is again used.
    assert_eq!(add(3, 2, C {c: 4}, 4), 13);

    let mut count_unique_calls = 0;
    assert_eq!(add2(1, 2, &mut count_unique_calls), 3);
    assert_eq!(count_unique_calls, 1);

    // Calling `add2` again won't increment `count_unique_calls`
    // because it's ignored as a parameter, and the other arguments
    // are the same.
    add2(1, 2, &mut count_unique_calls);
    assert_eq!(count_unique_calls, 1);
}
