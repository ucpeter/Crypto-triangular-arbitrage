/// Round a floating point number to 2 decimal places
pub fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

/// Round a floating point number to 4 decimal places
pub fn round4(val: f64) -> f64 {
    (val * 10_000.0).round() / 10_000.0
}

/// Convert string to f64 safely
pub fn parse_f64(s: &str) -> Option<f64> {
    match s.parse::<f64>() {
        Ok(v) if v.is_finite() => Some(v),
        _ => None,
    }
}
