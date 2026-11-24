/// Product of two measured lengths. Split from the surrounding add so the rounding stays the same.
#[inline]
pub fn mul(left: f64, right: f64) -> f64 {
    left * right
}

/// A count that fits in `u32`, as a length. Diagram sizes are far below that bound.
#[inline]
pub fn count(n: usize) -> f64 {
    f64::from(u32::try_from(n).expect("diagram count fits in u32"))
}

/// An integer that fits in `i32`, as a length. Every `i32` is an exact `f64`.
#[inline]
pub fn whole(n: i64) -> f64 {
    f64::from(i32::try_from(n).expect("diagram integer fits in i32"))
}

/// Truncate toward zero. `floor` then formatting keeps an exact integer.
pub fn trunc(value: f64) -> i64 {
    let integral = if value >= 0.0 { value.floor() } else { value.ceil() };
    let text = format!("{integral:.0}");
    text.parse().unwrap_or(0)
}

/// Two-decimal SVG number, matching Python `f"{v:.2f}"` then stripping trailing zeros.
#[must_use]
pub fn fmt(v: f64) -> String {
    let text = format!("{v:.2}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    if trimmed.is_empty() || trimmed == "-0" {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    }
}

/// Python `round` toward a whole number, using the same fixed-format rounding as `f"{v:.0f}"`.
#[must_use]
pub fn py_round(v: f64) -> i64 {
    let text = format!("{v:.0}");
    text.parse().unwrap_or(0)
}

pub fn py_fixed0(v: f64) -> String {
    format!("{v:.0}")
}

/// # Panics
/// Panics when the magnitude's base-10 exponent does not fit in `i32`.
#[must_use]
pub fn py_g(v: f64) -> String {
    if v == 0.0 {
        return "0".to_owned();
    }
    let exp = v.abs().log10().floor();
    let exp_i = i32::try_from(py_round(exp)).expect("exponent fits in i32");
    if (-4..6).contains(&exp_i) {
        let digits = usize::try_from((5 - exp_i).max(0)).expect("digit count fits in usize");
        let mut text = format!("{v:.digits$}");
        if text.contains('.') {
            while text.ends_with('0') {
                text.pop();
            }
            if text.ends_with('.') {
                text.pop();
            }
        }
        if text.is_empty() || text == "-" || text == "-0" {
            "0".to_owned()
        } else {
            text
        }
    } else {
        format!("{v:.6e}")
    }
}

pub fn py_repr(text: &str) -> String {
    let single = text.contains('\'');
    let double = text.contains('"');
    if single && !double {
        format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        format!("'{}'", text.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

pub fn py_channel(v: f64) -> u8 {
    let n = py_round(v).clamp(0, 255);
    u8::try_from(n).unwrap_or(0)
}
