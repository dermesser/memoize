use memoize::memoize;

// Patterns in memoized function arguments must be bound by name.
#[memoize]
fn manhattan_distance(_p1 @ (x1, y1): (i32, i32), _p2 @ (x2, y2): (i32, i32)) -> i32 {
    (x1 - x2).abs() + (y1 - y2).abs()
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum OnlyOne {
    Value(i32),
}

#[memoize]
fn get_value(_enum @ OnlyOne::Value(value): OnlyOne) -> i32 {
    value
}

fn main() {
    // `manhattan_distance` is only called once here.
    assert_eq!(manhattan_distance((1, 1), (1, 3)), 2);

    // Same with `get_value`.
    assert_eq!(get_value(OnlyOne::Value(0)), 0);
}
