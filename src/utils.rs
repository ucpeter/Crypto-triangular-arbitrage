pub fn round2(val: f64) -> f64 {
    (val * 100.0).round() / 100.0
}

pub fn round4(val: f64) -> f64 {
    (val * 10_000.0).round() / 10_000.0
}
