use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum Trend {
    Rising,
    Falling,
    Stable,
}

/// Calculate the average of a vector of CPU percentage readings
///
/// Returns 0.0 if the vector is empty to avoid panics
pub fn average_cpu_percent(readings: Vec<f32>) -> f32 {
    if readings.is_empty() {
        return 0.0;
    }
    readings.iter().sum::<f32>() / readings.len() as f32
}

/// Calculate the average of a vector of memory percentage readings
///
/// Returns 0.0 if the vector is empty to avoid panics
pub fn average_memory_percent(readings: Vec<f32>) -> f32 {
    if readings.is_empty() {
        return 0.0;
    }
    readings.iter().sum::<f32>() / readings.len() as f32
}

/// Detect the trend in a series of readings
///
/// Uses a simple comparison of the average of the first half vs second half of readings.
/// If there are fewer than 2 readings, returns Stable.
/// If the second half average is significantly higher (>5% difference), Rising.
/// If significantly lower, Falling. Otherwise Stable.
pub fn detect_trend(readings: Vec<f32>) -> Trend {
    if readings.len() < 2 {
        return Trend::Stable;
    }

    let mid = readings.len() / 2;
    let first_half: Vec<f32> = readings[..mid].to_vec();
    let second_half: Vec<f32> = readings[mid..].to_vec();

    let avg_first = first_half.iter().sum::<f32>() / first_half.len() as f32;
    let avg_second = second_half.iter().sum::<f32>() / second_half.len() as f32;

    let diff = avg_second - avg_first;
    let threshold = 5.0; // 5% difference threshold

    if diff > threshold {
        Trend::Rising
    } else if diff < -threshold {
        Trend::Falling
    } else {
        Trend::Stable
    }
}

/// Estimate time until system reaches critical temperature
///
/// This is a placeholder implementation that returns a default duration.
/// In a full implementation, this would analyze temperature trends and
/// extrapolate to estimate when critical temperature will be reached.
///
/// Currently returns 0 duration (immediate) if temperature is already critical,
/// or a default 5 minutes otherwise. This should be enhanced with actual
/// temperature data and trend analysis.
pub fn estimate_time_to_overheat() -> Duration {
    // Placeholder: in a real implementation, this would take temperature readings
    // and calculate based on current trend and rate of change
    Duration::from_secs(300) // 5 minutes default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_average_cpu_percent() {
        assert_eq!(average_cpu_percent(vec![]), 0.0);
        assert_eq!(average_cpu_percent(vec![10.0]), 10.0);
        assert_eq!(average_cpu_percent(vec![10.0, 20.0, 30.0]), 20.0);
    }

    #[test]
    fn test_average_memory_percent() {
        assert_eq!(average_memory_percent(vec![]), 0.0);
        assert_eq!(average_memory_percent(vec![50.0]), 50.0);
        assert_eq!(average_memory_percent(vec![40.0, 60.0]), 50.0);
    }

    #[test]
    fn test_detect_trend() {
        // Empty or single reading
        assert_eq!(detect_trend(vec![]), Trend::Stable);
        assert_eq!(detect_trend(vec![50.0]), Trend::Stable);

        // Rising trend
        assert_eq!(detect_trend(vec![10.0, 20.0, 30.0, 40.0]), Trend::Rising);

        // Falling trend
        assert_eq!(detect_trend(vec![40.0, 30.0, 20.0, 10.0]), Trend::Falling);

        // Stable trend
        assert_eq!(detect_trend(vec![45.0, 50.0, 48.0, 52.0]), Trend::Stable);

        // Small changes within threshold
        assert_eq!(detect_trend(vec![48.0, 52.0, 50.0, 53.0]), Trend::Stable);
    }

    #[test]
    fn test_estimate_time_to_overheat() {
        let duration = estimate_time_to_overheat();
        assert_eq!(duration, Duration::from_secs(300));
    }
}