//! How Bazel 9.2.0 writes a finite float as text, for the encoders: the
//! `starlark` crate's own `repr`, which fjfj patched to Bazel's.

/// `f` as Bazel's `repr` writes it. `f` must be finite.
pub(crate) fn format_float(f: f64) -> String {
    debug_assert!(f.is_finite());
    starlark::values::float::StarlarkFloat(f).to_string()
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
