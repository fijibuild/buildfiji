//! How Bazel 9.2.0 writes a finite float as text, for the encoders that
//! must not depend on the `starlark` crate's `repr`, which prints
//! `123456789.123` as `1.234568e+08`.
//!
//! The digits are the shortest that read back as the same float. They are
//! laid out positionally when the decimal exponent is from -4 to 16 and as
//! `d.ddde+XX` otherwise, always with a `.0` if they would read as an int.

/// `f` as Bazel's `repr` writes it. `f` must be finite.
pub(crate) fn format_float(f: f64) -> String {
    debug_assert!(f.is_finite());
    // Rust's `{:e}` gives the shortest digits: `-1.2345e-7`.
    let sci = format!("{f:e}");
    let (mantissa, exponent) = sci.split_once('e').expect("`{:e}` writes an exponent");
    let exponent: i32 = exponent.parse().expect("an integer exponent");
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    if (-4..17).contains(&exponent) {
        let point = exponent + 1;
        let body = if point <= 0 {
            format!("0.{}{}", "0".repeat((-point) as usize), digits)
        } else if digits.len() as i32 <= point {
            format!(
                "{digits}{}.0",
                "0".repeat((point - digits.len() as i32) as usize)
            )
        } else {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        };
        format!("{sign}{body}")
    } else {
        let mantissa = if digits.len() == 1 {
            digits
        } else {
            format!("{}.{}", &digits[..1], &digits[1..])
        };
        format!(
            "{sign}{mantissa}e{}{:02}",
            if exponent < 0 { '-' } else { '+' },
            exponent.abs()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::format_float;

    /// Each row is what Bazel 9.2.0's `repr` printed for the float.
    #[test]
    fn matches_bazel() {
        for (f, want) in [
            (1e-7, "1e-07"),
            (1e-5, "1e-05"),
            (1e-4, "0.0001"),
            (0.001, "0.001"),
            (0.00012345, "0.00012345"),
            (2.5e-5, "2.5e-05"),
            (123456.789, "123456.789"),
            (1e15, "1000000000000000.0"),
            (1e16, "10000000000000000.0"),
            (1e17, "1e+17"),
            (1e20, "1e+20"),
            (1.5e300, "1.5e+300"),
            (1e-300, "1e-300"),
            (1.7976931348623157e308, "1.7976931348623157e+308"),
            (0.1 + 0.2, "0.30000000000000004"),
            (100.0, "100.0"),
            (12345678901234567890.0, "1.2345678901234567e+19"),
            (1234567890123456.7, "1234567890123456.8"),
            (123456789012345678.0, "1.2345678901234568e+17"),
            (1.0 / 3.0, "0.3333333333333333"),
            (-1e-7, "-1e-07"),
            (3.0e10, "30000000000.0"),
            (0.5, "0.5"),
            (1.0, "1.0"),
            (1.5e16, "15000000000000000.0"),
            (9007199254740993.0, "9007199254740992.0"),
            (123456789.123, "123456789.123"),
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (-2.5, "-2.5"),
        ] {
            assert_eq!(format_float(f), want, "{f:e}");
        }
    }
}
