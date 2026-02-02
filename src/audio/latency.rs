use std::time::Duration;

/// Latency measurement tracker for audio system.
#[derive(Debug)]
pub struct LatencyMeasurement {
    samples: Vec<Duration>,
    max_samples: usize,
}

impl LatencyMeasurement {
    /// Create a new latency measurement tracker.
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::with_capacity(max_samples),
            max_samples,
        }
    }

    /// Record a latency measurement.
    pub fn record(&mut self, latency: Duration) {
        if self.samples.len() >= self.max_samples {
            self.samples.remove(0);
        }
        self.samples.push(latency);
    }

    /// Get the average latency.
    pub fn average(&self) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.samples.iter().sum();
        total / self.samples.len() as u32
    }

    /// Get a specific percentile (0-100) of the latency measurements.
    pub fn percentile(&self, p: f64) -> Duration {
        if self.samples.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted = self.samples.clone();
        sorted.sort();

        let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    /// Get the minimum recorded latency.
    pub fn min(&self) -> Duration {
        self.samples.iter().min().copied().unwrap_or(Duration::ZERO)
    }

    /// Get the maximum recorded latency.
    pub fn max(&self) -> Duration {
        self.samples.iter().max().copied().unwrap_or(Duration::ZERO)
    }

    /// Get the number of recorded samples.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Clear all recorded samples.
    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Get average latency in milliseconds.
    pub fn average_ms(&self) -> f64 {
        self.average().as_secs_f64() * 1000.0
    }
}

impl Default for LatencyMeasurement {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_measurement() {
        let mut measurement = LatencyMeasurement::new(5);

        measurement.record(Duration::from_millis(10));
        measurement.record(Duration::from_millis(20));
        measurement.record(Duration::from_millis(30));

        assert_eq!(measurement.sample_count(), 3);
        assert_eq!(measurement.average(), Duration::from_millis(20));
        assert_eq!(measurement.min(), Duration::from_millis(10));
        assert_eq!(measurement.max(), Duration::from_millis(30));
    }

    #[test]
    fn test_latency_percentile() {
        let mut measurement = LatencyMeasurement::new(10);

        for i in 1..=10 {
            measurement.record(Duration::from_millis(i * 10));
        }

        // 50th percentile should be around 50-60ms
        let p50 = measurement.percentile(50.0);
        assert!(p50 >= Duration::from_millis(50) && p50 <= Duration::from_millis(60));

        // 90th percentile should be around 90-100ms
        let p90 = measurement.percentile(90.0);
        assert!(p90 >= Duration::from_millis(90) && p90 <= Duration::from_millis(100));
    }

    #[test]
    fn test_max_samples() {
        let mut measurement = LatencyMeasurement::new(3);

        measurement.record(Duration::from_millis(10));
        measurement.record(Duration::from_millis(20));
        measurement.record(Duration::from_millis(30));
        measurement.record(Duration::from_millis(40));

        // Should only have 3 samples, oldest removed
        assert_eq!(measurement.sample_count(), 3);
        assert_eq!(measurement.min(), Duration::from_millis(20));
    }
}
