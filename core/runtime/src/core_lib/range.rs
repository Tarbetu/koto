//! The `range` core library module

use crate::prelude::*;

/// Initializes the `range` core library module
pub fn make_module() -> ValueMap {
    use Value::*;

    let result = ValueMap::with_type("core.range");

    result.add_fn("contains", |ctx| {
        let expected_error = "a Range, and a Number or another Range";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), [Number(n)]) => Ok(r.contains(*n).into()),
            (Range(a), [Range(b)]) => {
                let r_a = a.as_sorted_range();
                let r_b = b.as_sorted_range();
                let result = r_b.start >= r_a.start && r_b.end <= r_a.end;
                Ok(result.into())
            }
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("end", |ctx| {
        let expected_error = "a Range";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), []) => Ok(r.end().map_or(Null, |(end, _inclusive)| end.into())),
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("expanded", |ctx| {
        let expected_error = "a Range and Number";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), [Number(n)]) => match (r.start(), r.end()) {
                (Some(start), Some((end, inclusive))) => {
                    let n = i64::from(n);
                    let result = if r.is_ascending() {
                        IntRange::bounded(start - n, end + n, inclusive)
                    } else {
                        IntRange::bounded(start + n, end - n, inclusive)
                    };
                    Ok(result.into())
                }
                _ => runtime_error!("range.expanded can't be used with '{r}'"),
            },
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("intersection", |ctx| {
        let expected_error = "two Ranges";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(a), [Range(b)]) => Ok(a.intersection(b).map_or(Null, |result| result.into())),
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("is_inclusive", |ctx| {
        let expected_error = "a Range";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), []) => Ok(r.end().map_or(false, |(_end, inclusive)| inclusive).into()),
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("size", |ctx| {
        let expected_error = "a Range";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), []) => match r.size() {
                Some(size) => Ok(size.into()),
                None => runtime_error!("range.size can't be used with '{r}'"),
            },
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("start", |ctx| {
        let expected_error = "a Range";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), []) => Ok(r.start().map_or(Null, Value::from)),
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result.add_fn("union", |ctx| {
        let expected_error = "a Range, and a Number or another Range";

        match ctx.instance_and_args(is_range, expected_error)? {
            (Range(r), [Number(n)]) => {
                let n = i64::from(n);
                match (r.start(), r.end()) {
                    (Some(start), Some((end, inclusive))) => {
                        let result = if start <= end {
                            IntRange::bounded(start.min(n), end.max(n + 1), inclusive)
                        } else {
                            IntRange::bounded(start.max(n), end.min(n - 1), inclusive)
                        };
                        Ok(result.into())
                    }
                    _ => runtime_error!("range.union can't be used with '{r}'"),
                }
            }
            (Range(a), [Range(b)]) => match (a.start(), a.end()) {
                (Some(start), Some((end, inclusive))) => {
                    let r_b = b.as_sorted_range();
                    let result = if start <= end {
                        IntRange::bounded(start.min(r_b.start), end.max(r_b.end), inclusive)
                    } else {
                        IntRange::bounded(start.max(r_b.end - 1), end.min(r_b.start), inclusive)
                    };
                    Ok(result.into())
                }
                _ => runtime_error!("range.union can't be used with '{a}' and '{b}'"),
            },
            (_, unexpected) => type_error_with_slice(expected_error, unexpected),
        }
    });

    result
}

fn is_range(value: &Value) -> bool {
    matches!(value, Value::Range(_))
}
