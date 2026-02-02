use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Statistics for a profiled section.
#[derive(Debug, Clone, Default)]
pub struct SectionStats {
    pub total_time: Duration,
    pub call_count: u64,
    pub max_time: Duration,
    pub min_time: Duration,
}

impl SectionStats {
    #[cfg_attr(not(feature = "profiling"), allow(dead_code))]
    fn new() -> Self {
        Self {
            total_time: Duration::ZERO,
            call_count: 0,
            max_time: Duration::ZERO,
            min_time: Duration::MAX,
        }
    }

    #[cfg_attr(not(feature = "profiling"), allow(dead_code))]
    fn record(&mut self, duration: Duration) {
        self.total_time += duration;
        self.call_count += 1;
        self.max_time = self.max_time.max(duration);
        self.min_time = self.min_time.min(duration);
    }

    /// Get the average time per call in milliseconds.
    pub fn avg_ms(&self) -> f64 {
        if self.call_count == 0 {
            return 0.0;
        }
        self.total_time.as_secs_f64() * 1000.0 / self.call_count as f64
    }
}

/// Frame-based performance profiler.
#[derive(Debug, Default)]
pub struct FrameProfiler {
    sections: HashMap<&'static str, SectionStats>,
    frame_times: Vec<Duration>,
    last_frame_start: Option<Instant>,
    max_frame_history: usize,
}

impl FrameProfiler {
    /// Create a new profiler.
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            frame_times: Vec::with_capacity(300),
            last_frame_start: None,
            max_frame_history: 300, // 5 seconds at 60fps
        }
    }

    /// Start a new frame.
    pub fn begin_frame(&mut self) {
        if let Some(start) = self.last_frame_start.take() {
            let duration = start.elapsed();
            self.frame_times.push(duration);

            // Keep only recent frames
            if self.frame_times.len() > self.max_frame_history {
                self.frame_times.remove(0);
            }
        }
        self.last_frame_start = Some(Instant::now());
    }

    /// Time a section of code.
    #[cfg(feature = "profiling")]
    pub fn time<F, R>(&mut self, name: &'static str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();

        let stats = self.sections.entry(name).or_insert_with(SectionStats::new);
        stats.record(duration);

        result
    }

    /// Time a section of code (no-op when profiling is disabled).
    #[cfg(not(feature = "profiling"))]
    #[inline(always)]
    pub fn time<F, R>(&mut self, _name: &'static str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }

    /// Get the average FPS over the recorded history.
    pub fn get_fps(&self) -> f64 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let total: Duration = self.frame_times.iter().sum();
        if total.is_zero() {
            return 0.0;
        }

        self.frame_times.len() as f64 / total.as_secs_f64()
    }

    /// Get the average frame time in milliseconds.
    pub fn get_frame_time_ms(&self) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }

        let total: Duration = self.frame_times.iter().sum();
        total.as_secs_f64() * 1000.0 / self.frame_times.len() as f64
    }

    /// Get all section statistics.
    pub fn sections(&self) -> &HashMap<&'static str, SectionStats> {
        &self.sections
    }

    /// Reset all statistics.
    pub fn reset(&mut self) {
        self.sections.clear();
        self.frame_times.clear();
        self.last_frame_start = None;
    }

    /// Draw debug overlay (only when profiling feature is enabled).
    #[cfg(feature = "profiling")]
    pub fn draw_debug(&self) {
        use macroquad::prelude::*;

        let fps = self.get_fps();
        let frame_ms = self.get_frame_time_ms();

        // Background
        draw_rectangle(
            5.0,
            5.0,
            250.0,
            20.0 + self.sections.len() as f32 * 16.0,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        // FPS and frame time
        draw_text(
            &format!("FPS: {:.1} ({:.2}ms)", fps, frame_ms),
            10.0,
            20.0,
            16.0,
            WHITE,
        );

        // Section times
        let mut y = 40.0;
        let mut sorted_sections: Vec<_> = self.sections.iter().collect();
        sorted_sections.sort_by(|a, b| b.1.avg_ms().partial_cmp(&a.1.avg_ms()).unwrap());

        for (name, stats) in sorted_sections {
            let color = if stats.avg_ms() > 5.0 {
                RED
            } else if stats.avg_ms() > 2.0 {
                YELLOW
            } else {
                GREEN
            };

            draw_text(
                &format!(
                    "{}: {:.2}ms (max: {:.2}ms)",
                    name,
                    stats.avg_ms(),
                    stats.max_time.as_secs_f64() * 1000.0
                ),
                10.0,
                y,
                14.0,
                color,
            );
            y += 16.0;
        }
    }

    /// Draw debug overlay (no-op when profiling is disabled).
    #[cfg(not(feature = "profiling"))]
    #[inline(always)]
    pub fn draw_debug(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_stats() {
        let mut stats = SectionStats::new();
        stats.record(Duration::from_millis(10));
        stats.record(Duration::from_millis(20));

        assert_eq!(stats.call_count, 2);
        assert_eq!(stats.total_time, Duration::from_millis(30));
        assert_eq!(stats.max_time, Duration::from_millis(20));
        assert_eq!(stats.min_time, Duration::from_millis(10));
        assert!((stats.avg_ms() - 15.0).abs() < 0.001);
    }

    #[test]
    fn test_frame_profiler() {
        let mut profiler = FrameProfiler::new();

        // Simulate frames
        for _ in 0..10 {
            profiler.begin_frame();
            std::thread::sleep(Duration::from_millis(1));
        }

        // Should have recorded frames
        assert!(profiler.frame_times.len() > 0);
        assert!(profiler.get_fps() > 0.0);
    }

    #[test]
    fn test_profiler_time() {
        let mut profiler = FrameProfiler::new();

        let result = profiler.time("test_section", || {
            std::thread::sleep(Duration::from_millis(1));
            42
        });

        assert_eq!(result, 42);

        #[cfg(feature = "profiling")]
        {
            let stats = profiler.sections().get("test_section");
            assert!(stats.is_some());
            assert_eq!(stats.unwrap().call_count, 1);
        }
    }
}
