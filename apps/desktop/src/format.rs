use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_ms() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |d| d.as_millis() as f64)
}

/// e.g. "42m", "3h 20m" or "2d 4h", as the app's `formatDuration`.
pub fn duration(ms: f64) -> String {
    let minutes = ((ms / 60_000.0).round() as i64).max(1);
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = minutes / 60;
    if hours < 24 {
        return match minutes % 60 {
            0 => format!("{hours}h"),
            rest => format!("{hours}h {rest}m"),
        };
    }
    match hours % 24 {
        0 => format!("{}d", hours / 24),
        rest => format!("{}d {rest}h", hours / 24),
    }
}

#[cfg(test)]
mod tests {
    use super::duration;

    #[test]
    fn rounds_to_the_two_largest_units() {
        assert_eq!(duration(0.0), "1m");
        assert_eq!(duration(42.0 * 60_000.0), "42m");
        assert_eq!(duration(200.0 * 60_000.0), "3h 20m");
        assert_eq!(duration(3.0 * 3_600_000.0), "3h");
        assert_eq!(duration(52.0 * 3_600_000.0), "2d 4h");
    }
}
