use bms_model::bms_model::BMSModel;
use bms_model::layer::{EventType, Layer};

use crate::bga::bg_image_processor::BGImageProcessor;

/// Movie file extensions supported for BGA
pub static MOV_EXTENSION: &[&str] = &[
    "mp4", "wmv", "m4v", "webm", "mpg", "mpeg", "m1v", "m2v", "avi",
];

/// Lightweight BGA timeline entry extracted from BMSModel timelines.
/// Stores only the BGA-relevant fields (time, bga id, layer id, event layers).
struct BgaTimeline {
    /// Timeline time in milliseconds (matching Java TimeLine.getTime())
    time_ms: i64,
    /// BGA id (-1 = no change, -2 = stop)
    bga: i32,
    /// Layer id (-1 = no change, -2 = stop)
    layer: i32,
    /// Event layers (POOR layer etc.)
    eventlayer: Vec<Layer>,
}

/// BGA resource manager and renderer
pub struct BGAProcessor {
    progress: f32,
    /// Currently playing BGA id
    playingbgaid: i32,
    /// Currently playing layer id
    playinglayerid: i32,
    /// Miss layer display start time
    misslayertime: i64,
    get_misslayer_duration: i64,
    /// Current miss layer sequence
    misslayer: Option<Layer>,
    /// Current time in milliseconds (matching Java BGAProcessor.time)
    time: i64,
    cache: Option<BGImageProcessor>,
    /// Filtered timelines containing BGA/layer/eventlayer data
    timelines: Vec<BgaTimeline>,
    pos: usize,
    rbga: bool,
    rlayer: bool,
}

impl Default for BGAProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl BGAProcessor {
    pub fn new() -> Self {
        BGAProcessor {
            progress: 0.0,
            playingbgaid: -1,
            playinglayerid: -1,
            misslayertime: 0,
            get_misslayer_duration: 0,
            misslayer: None,
            time: 0,
            cache: Some(BGImageProcessor::new(256, 1)),
            timelines: Vec::new(),
            pos: 0,
            rbga: false,
            rlayer: false,
        }
    }

    /// Create a BGAProcessor and load timeline data from the given model.
    /// Convenience constructor for testing.
    pub fn from_model(model: &BMSModel) -> Self {
        let mut proc = Self::new();
        proc.set_model_timelines(model);
        proc
    }

    /// Extract and store BGA-relevant timelines from the model.
    /// Corresponds to the timeline-filtering part of Java BGAProcessor.setModel().
    pub fn set_model_timelines(&mut self, model: &BMSModel) {
        self.progress = 0.0;
        if let Some(ref mut cache) = self.cache {
            cache.clear();
        }
        self.reset_currently_playing_bga();

        let mut tls = Vec::new();
        for tl in model.get_all_time_lines() {
            if tl.get_bga() != -1 || tl.get_layer() != -1 || !tl.get_eventlayer().is_empty() {
                tls.push(BgaTimeline {
                    // Java TimeLine.getTime() returns (int)(time / 1000) i.e. milliseconds
                    time_ms: tl.get_time() as i64,
                    bga: tl.get_bga(),
                    layer: tl.get_layer(),
                    eventlayer: tl.get_eventlayer().to_vec(),
                });
            }
        }
        self.timelines = tls;

        self.progress = 1.0;
    }

    pub fn set_model(&mut self, _model_path: Option<&str>) {
        self.progress = 0.0;
        if let Some(ref mut cache) = self.cache {
            cache.clear();
        }
        self.reset_currently_playing_bga();

        // TODO: Resource loading (images/movies) requires file I/O + MovieProcessor
        // Timeline loading is done separately via set_model_timelines()

        self.progress = 1.0;
    }

    pub fn abort(&mut self) {
        self.progress = 1.0;
    }

    pub fn dispose_old(&mut self) {
        if let Some(ref mut cache) = self.cache {
            cache.dispose_old();
        }
    }

    pub fn prepare(&mut self, _player: &dyn std::any::Any) {
        self.pos = 0;
        // Java: cache.prepare(timelines) — skipped for now (resource pre-caching)
        self.reset_currently_playing_bga();
        self.time = 0;
    }

    fn reset_currently_playing_bga(&mut self) {
        self.playingbgaid = -1;
        self.playinglayerid = -1;
        self.misslayertime = 0;
        self.misslayer = None;
    }

    /// Update BGA state to the given time (microseconds).
    /// Public API using project-standard microsecond timing.
    pub fn update(&mut self, time_us: i64) {
        // Convert to milliseconds for internal comparison (matching Java)
        self.prepare_bga(time_us / 1000);
    }

    /// Scan timelines and update playingbgaid/playinglayerid/misslayer.
    /// Corresponds to Java BGAProcessor.prepareBGA(long time) where time is in ms.
    pub fn prepare_bga(&mut self, time: i64) {
        if time < 0 {
            self.time = -1;
            return;
        }
        for i in self.pos..self.timelines.len() {
            let tl = &self.timelines[i];
            if tl.time_ms > time {
                break;
            }

            if tl.time_ms > self.time {
                let bga = tl.bga;
                if bga == -2 {
                    self.playingbgaid = -1;
                    self.rbga = false;
                } else if bga >= 0 {
                    self.playingbgaid = bga;
                    self.rbga = false;
                }

                let layer = tl.layer;
                if layer == -2 {
                    self.playinglayerid = -1;
                    self.rlayer = false;
                } else if layer >= 0 {
                    self.playinglayerid = layer;
                    self.rlayer = false;
                }

                let eventlayer = &tl.eventlayer;
                for poor in eventlayer {
                    if poor.event.event_type == EventType::Miss {
                        self.misslayer = Some(poor.clone());
                    }
                }
            } else {
                self.pos += 1;
            }
        }

        self.time = time;
    }

    pub fn draw_bga(&self) {
        // TODO: Requires SkinBGA, SkinObjectRenderer, Texture rendering
        // In Java, this draws the current BGA frame, layer, or miss layer
    }

    pub fn set_misslayer_tme(&mut self, time: i64) {
        self.misslayertime = time;
        // TODO: getMisslayerDuration from PlayerConfig
        self.get_misslayer_duration = 500;
    }

    pub fn stop(&mut self) {
        // TODO: stop all MovieProcessors
    }

    pub fn dispose(&mut self) {
        if let Some(ref mut cache) = self.cache {
            cache.dispose();
        }
    }

    pub fn get_progress(&self) -> f32 {
        self.progress
    }

    /// Get currently playing BGA id.
    pub fn current_bga_id(&self) -> i32 {
        self.playingbgaid
    }

    /// Get currently playing layer id.
    pub fn current_layer_id(&self) -> i32 {
        self.playinglayerid
    }

    /// Get BGA texture data for the given id at the specified time.
    /// Corresponds to Java getBGAData(long time, int id, boolean cont).
    fn get_bga_data(&self, _time: i64, id: i32, _cont: bool) -> Option<()> {
        if self.progress != 1.0 || id == -1 {
            return None;
        }
        // TODO: Requires MovieProcessor array and BGImageProcessor cache
        None
    }

    /// Draw BGA with fixed aspect ratio.
    /// Corresponds to Java drawBGAFixRatio(SkinBGA dst, SkinObjectRenderer sprite, Rectangle r, Texture bga).
    fn draw_bga_fix_ratio(&self) {
        // TODO: Requires SkinBGA, SkinObjectRenderer, Texture, Rectangle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bms_model::time_line::TimeLine;

    /// Helper: create a BMSModel with BGA timelines from (time_us, bga, layer) tuples.
    /// bga/layer values are set directly (including -2 for stop).
    fn model_with_bga_timelines(entries: &[(i64, i32, i32)]) -> BMSModel {
        let mut model = BMSModel::new();
        let mut timelines = Vec::new();
        for &(time_us, bga, layer) in entries {
            let mut tl = TimeLine::new(0.0, time_us, 18);
            tl.set_bga(bga);
            tl.set_layer(layer);
            timelines.push(tl);
        }
        model.set_all_time_line(timelines);
        model
    }

    #[test]
    fn test_empty_model() {
        let model = BMSModel::new();
        let mut proc = BGAProcessor::from_model(&model);
        assert_eq!(proc.current_bga_id(), -1);
        assert_eq!(proc.current_layer_id(), -1);

        proc.update(1_000_000); // 1 second
        assert_eq!(proc.current_bga_id(), -1);
        assert_eq!(proc.current_layer_id(), -1);
    }

    #[test]
    fn test_single_bga_event() {
        // BGA id 5 at time 1000ms (1_000_000 μs)
        let model = model_with_bga_timelines(&[(1_000_000, 5, -1)]);
        let mut proc = BGAProcessor::from_model(&model);

        // Before the event
        proc.update(500_000); // 500ms
        assert_eq!(proc.current_bga_id(), -1);

        // At the event time
        proc.update(1_000_000); // 1000ms
        assert_eq!(proc.current_bga_id(), 5);

        // After the event
        proc.update(2_000_000); // 2000ms
        assert_eq!(proc.current_bga_id(), 5);
    }

    #[test]
    fn test_bga_stop_event() {
        // BGA id 3 at 1s, BGA stop (-2) at 2s
        let model = model_with_bga_timelines(&[(1_000_000, 3, -1), (2_000_000, -2, -1)]);

        let mut proc = BGAProcessor::from_model(&model);

        proc.update(1_500_000); // 1500ms — should see BGA 3
        assert_eq!(proc.current_bga_id(), 3);

        proc.update(2_500_000); // 2500ms — BGA stopped
        assert_eq!(proc.current_bga_id(), -1);
    }

    #[test]
    fn test_layer_events() {
        let model = model_with_bga_timelines(&[(500_000, -1, 10), (1_500_000, -1, 20)]);

        let mut proc = BGAProcessor::from_model(&model);

        proc.update(0);
        assert_eq!(proc.current_layer_id(), -1);

        proc.update(500_000);
        assert_eq!(proc.current_layer_id(), 10);

        proc.update(2_000_000);
        assert_eq!(proc.current_layer_id(), 20);
    }

    #[test]
    fn test_bga_and_layer_combined() {
        let model = model_with_bga_timelines(&[(1_000_000, 5, 10)]);

        let mut proc = BGAProcessor::from_model(&model);
        proc.update(1_000_000);
        assert_eq!(proc.current_bga_id(), 5);
        assert_eq!(proc.current_layer_id(), 10);
    }

    #[test]
    fn test_multiple_bga_changes() {
        let entries: Vec<(i64, i32, i32)> = (0..5)
            .map(|i| ((i + 1) * 1_000_000, i as i32, -1))
            .collect();
        let model = model_with_bga_timelines(&entries);

        let mut proc = BGAProcessor::from_model(&model);

        // Step through each second
        for i in 0..5 {
            proc.update((i + 1) * 1_000_000);
            assert_eq!(proc.current_bga_id(), i as i32);
        }
    }

    #[test]
    fn test_negative_time() {
        let model = model_with_bga_timelines(&[(1_000_000, 5, -1)]);
        let mut proc = BGAProcessor::from_model(&model);

        proc.update(-1_000_000); // negative time
        assert_eq!(proc.current_bga_id(), -1);

        // After negative time, positive time should still work
        proc.update(1_000_000);
        assert_eq!(proc.current_bga_id(), 5);
    }

    #[test]
    fn test_prepare_resets_state() {
        let model = model_with_bga_timelines(&[(1_000_000, 5, -1)]);

        let mut proc = BGAProcessor::from_model(&model);
        proc.update(1_000_000);
        assert_eq!(proc.current_bga_id(), 5);

        // Reset via prepare()
        proc.prepare(&());
        assert_eq!(proc.current_bga_id(), -1);

        // Should be able to replay
        proc.update(1_000_000);
        assert_eq!(proc.current_bga_id(), 5);
    }
}
