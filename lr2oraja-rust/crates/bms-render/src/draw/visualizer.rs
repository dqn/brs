// Visualizer draw logic — pixel computation for BPM graph, hit error,
// note distribution, timing distribution, and timing visualizer.

/// RGBA color.
#[derive(Copy, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Buffer dimensions.
#[derive(Copy, Clone)]
struct Dimensions {
    w: u32,
    h: u32,
}

impl Dimensions {
    const fn new(w: u32, h: u32) -> Self {
        Self { w, h }
    }
}

/// Set a single RGBA pixel in the buffer.
fn set_pixel(buf: &mut [u8], x: u32, y: u32, dim: Dimensions, color: Color) {
    let Dimensions { w, h } = dim;
    let Color { r, g, b, a } = color;
    if x >= w || y >= h {
        return;
    }
    let idx = ((y * w + x) * 4) as usize;
    if idx + 3 < buf.len() {
        buf[idx] = r;
        buf[idx + 1] = g;
        buf[idx + 2] = b;
        buf[idx + 3] = a;
    }
}

/// Draw a horizontal line in the buffer.
fn draw_hline(buf: &mut [u8], x0: u32, x1: u32, y: u32, dim: Dimensions, color: Color) {
    let start = x0.min(x1);
    let end = x0.max(x1);
    for x in start..=end.min(dim.w.saturating_sub(1)) {
        set_pixel(buf, x, y, dim, color);
    }
}

/// Draw a vertical line in the buffer.
fn draw_vline(buf: &mut [u8], x: u32, y0: u32, y1: u32, dim: Dimensions, color: Color) {
    let start = y0.min(y1);
    let end = y0.max(y1);
    for y in start..=end.min(dim.h.saturating_sub(1)) {
        set_pixel(buf, x, y, dim, color);
    }
}

/// Draw a filled rectangle in the buffer.
fn fill_rect(
    buf: &mut [u8],
    x0: u32,
    y0: u32,
    rect_w: u32,
    rect_h: u32,
    dim: Dimensions,
    color: Color,
) {
    for dy in 0..rect_h {
        for dx in 0..rect_w {
            set_pixel(buf, x0 + dx, y0 + dy, dim, color);
        }
    }
}

/// Compute BPM graph pixel data.
///
/// Draws a line graph where X = time, Y = BPM (scaled to min-max range).
/// White line on transparent background. Returns RGBA pixel data.
pub fn compute_bpm_graph_pixels(bpm_events: &[(i64, f64)], width: u32, height: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    if width == 0 || height == 0 || bpm_events.is_empty() {
        return buf;
    }

    let min_bpm = bpm_events
        .iter()
        .map(|(_, b)| *b)
        .fold(f64::INFINITY, f64::min);
    let max_bpm = bpm_events
        .iter()
        .map(|(_, b)| *b)
        .fold(f64::NEG_INFINITY, f64::max);
    let min_time = bpm_events.iter().map(|(t, _)| *t).min().unwrap_or(0);
    let max_time = bpm_events.iter().map(|(t, _)| *t).max().unwrap_or(0);

    let bpm_range = if (max_bpm - min_bpm).abs() < 1e-9 {
        1.0
    } else {
        max_bpm - min_bpm
    };
    let time_range = if max_time == min_time {
        1i64
    } else {
        max_time - min_time
    };

    let to_x =
        |t: i64| -> u32 { ((t - min_time) as f64 / time_range as f64 * (width - 1) as f64) as u32 };
    let to_y = |bpm: f64| -> u32 {
        let normalized = (bpm - min_bpm) / bpm_range;
        // Invert Y: top = max BPM
        ((1.0 - normalized) * (height - 1) as f64) as u32
    };

    // Draw line segments between consecutive BPM events (step function style)
    for i in 0..bpm_events.len() {
        let (t, bpm) = bpm_events[i];
        let x = to_x(t);
        let y = to_y(bpm);

        if i + 1 < bpm_events.len() {
            let (t_next, _) = bpm_events[i + 1];
            let x_next = to_x(t_next);
            // Horizontal line at current BPM until next event
            draw_hline(
                &mut buf,
                x,
                x_next,
                y,
                Dimensions::new(width, height),
                Color::new(255, 255, 255, 255),
            );
            // Vertical line at transition point
            let y_next = to_y(bpm_events[i + 1].1);
            draw_vline(
                &mut buf,
                x_next,
                y,
                y_next,
                Dimensions::new(width, height),
                Color::new(255, 255, 255, 255),
            );
        } else {
            // Last event: draw to the end
            draw_hline(
                &mut buf,
                x,
                width - 1,
                y,
                Dimensions::new(width, height),
                Color::new(255, 255, 255, 255),
            );
        }
    }

    buf
}

/// Compute hit error visualizer pixel data.
///
/// Draws a histogram centered at 0. Color: green center, yellow edges, red far edges.
/// Returns RGBA pixel data.
pub fn compute_hit_error_pixels(errors_us: &[i64], width: u32, height: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    if width == 0 || height == 0 || errors_us.is_empty() {
        return buf;
    }

    // Determine the range for the histogram
    let max_abs = errors_us
        .iter()
        .map(|e| e.unsigned_abs())
        .max()
        .unwrap_or(1)
        .max(1);

    // Number of bins = width
    let num_bins = width as usize;
    let mut bins = vec![0u32; num_bins];

    for &err in errors_us {
        // Map error from [-max_abs, max_abs] to [0, num_bins-1]
        let normalized = (err as f64 + max_abs as f64) / (2.0 * max_abs as f64);
        let bin = (normalized * (num_bins - 1) as f64) as usize;
        let bin = bin.min(num_bins - 1);
        bins[bin] += 1;
    }

    let max_count = *bins.iter().max().unwrap_or(&1).max(&1);

    let center = width / 2;

    for (i, &count) in bins.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let x = i as u32;
        let bar_height = (count as f64 / max_count as f64 * (height - 1) as f64) as u32;

        // Color by distance from center
        let dist_from_center = x.abs_diff(center);
        let ratio = dist_from_center as f64 / center.max(1) as f64;
        let (r, g, b) = if ratio < 0.33 {
            (0u8, 255u8, 0u8) // Green center
        } else if ratio < 0.66 {
            (255, 255, 0) // Yellow edges
        } else {
            (255, 0, 0) // Red far edges
        };

        // Draw bar from bottom up
        for dy in 0..=bar_height {
            let y = height - 1 - dy;
            set_pixel(
                &mut buf,
                x,
                y,
                Dimensions::new(width, height),
                Color::new(r, g, b, 255),
            );
        }
    }

    buf
}

/// Compute note distribution graph pixel data.
///
/// Draws vertical bars per lane. Blue bars on transparent background.
/// Returns RGBA pixel data.
pub fn compute_note_distribution_pixels(lane_counts: &[u32], width: u32, height: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    if width == 0 || height == 0 || lane_counts.is_empty() {
        return buf;
    }

    let num_lanes = lane_counts.len() as u32;
    let bar_width = (width / num_lanes).max(1);
    let max_count = *lane_counts.iter().max().unwrap_or(&1).max(&1);

    for (i, &count) in lane_counts.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let bar_height = (count as f64 / max_count as f64 * (height - 1) as f64) as u32;
        let x0 = i as u32 * bar_width;
        let y0 = height - 1 - bar_height;

        fill_rect(
            &mut buf,
            x0,
            y0,
            bar_width.min(width - x0),
            bar_height + 1,
            Dimensions::new(width, height),
            Color::new(64, 128, 255, 255),
        );
    }

    buf
}

/// Judge colors: PG=cyan, GR=yellow, GD=green, BD=magenta, PR=red.
const JUDGE_COLORS: [(u8, u8, u8); 5] = [
    (0, 255, 255), // PG - cyan
    (255, 255, 0), // GR - yellow
    (0, 255, 0),   // GD - green
    (255, 0, 255), // BD - magenta
    (255, 0, 0),   // PR - red
];

/// Compute timing distribution graph pixel data.
///
/// Draws a horizontal bar chart. Colors: PG=cyan, GR=yellow, GD=green, BD=magenta, PR=red.
/// Returns RGBA pixel data.
pub fn compute_timing_distribution_pixels(
    judge_counts: &[u32; 5],
    width: u32,
    height: u32,
) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    if width == 0 || height == 0 {
        return buf;
    }

    let max_count = *judge_counts.iter().max().unwrap_or(&1).max(&1);
    let bar_height = (height / 5).max(1);
    let gap = if bar_height > 2 { 1 } else { 0 };

    for (i, &count) in judge_counts.iter().enumerate() {
        if count == 0 {
            continue;
        }
        let bar_width = (count as f64 / max_count as f64 * (width - 1) as f64) as u32;
        let y0 = i as u32 * bar_height;
        let (r, g, b) = JUDGE_COLORS[i];

        fill_rect(
            &mut buf,
            0,
            y0,
            bar_width + 1,
            bar_height.saturating_sub(gap),
            Dimensions::new(width, height),
            Color::new(r, g, b, 255),
        );
    }

    buf
}

/// Compute timing visualizer pixel data.
///
/// Draws a scatter plot: X = time, Y = error. Center line at Y=0 (perfect timing).
/// Dots colored by error magnitude. Returns RGBA pixel data.
pub fn compute_timing_visualizer_pixels(
    timing_data: &[(i64, i64)],
    width: u32,
    height: u32,
) -> Vec<u8> {
    let mut buf = vec![0u8; (width * height * 4) as usize];
    if width == 0 || height == 0 || timing_data.is_empty() {
        return buf;
    }

    let min_time = timing_data.iter().map(|(t, _)| *t).min().unwrap_or(0);
    let max_time = timing_data.iter().map(|(t, _)| *t).max().unwrap_or(0);
    let max_error = timing_data
        .iter()
        .map(|(_, e)| e.unsigned_abs())
        .max()
        .unwrap_or(1)
        .max(1);

    let time_range = if max_time == min_time {
        1i64
    } else {
        max_time - min_time
    };
    let center_y = height / 2;

    // Draw center line (dim white)
    draw_hline(
        &mut buf,
        0,
        width - 1,
        center_y,
        Dimensions::new(width, height),
        Color::new(80, 80, 80, 255),
    );

    for &(time, error) in timing_data {
        let x = ((time - min_time) as f64 / time_range as f64 * (width - 1) as f64) as u32;
        // Map error to Y: negative (early) = above center, positive (late) = below center
        let y_offset = (error as f64 / max_error as f64 * center_y as f64) as i32;
        let y = (center_y as i32 + y_offset).clamp(0, (height - 1) as i32) as u32;

        // Color by error magnitude: green (small) → yellow → red (large)
        let ratio = error.unsigned_abs() as f64 / max_error as f64;
        let (r, g, b) = if ratio < 0.33 {
            (0u8, 255u8, 0u8)
        } else if ratio < 0.66 {
            (255, 255, 0)
        } else {
            (255, 0, 0)
        };

        // Draw a 3x3 dot for visibility
        for dy in 0..3u32 {
            for dx in 0..3u32 {
                set_pixel(
                    &mut buf,
                    x.saturating_sub(1) + dx,
                    y.saturating_sub(1) + dy,
                    Dimensions::new(width, height),
                    Color::new(r, g, b, 255),
                );
            }
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Check if any pixel in the buffer is non-transparent.
    fn has_visible_pixels(buf: &[u8]) -> bool {
        buf.chunks(4).any(|px| px[3] > 0)
    }

    /// Count non-transparent pixels in the buffer.
    fn count_visible_pixels(buf: &[u8]) -> usize {
        buf.chunks(4).filter(|px| px[3] > 0).count()
    }

    #[test]
    fn bpm_graph_empty_events() {
        let buf = compute_bpm_graph_pixels(&[], 100, 50);
        assert_eq!(buf.len(), 100 * 50 * 4);
        assert!(!has_visible_pixels(&buf));
    }

    #[test]
    fn bpm_graph_single_bpm_horizontal_line() {
        let events = vec![(0, 120.0)];
        let buf = compute_bpm_graph_pixels(&events, 100, 50);
        assert_eq!(buf.len(), 100 * 50 * 4);
        assert!(has_visible_pixels(&buf));

        // With a single BPM, the line should be at y = height/2 (normalized to 0.5 with range 1.0)
        // Actually with single BPM, bpm_range = 1.0, normalized = (120-120)/1 = 0,
        // so y = (1.0 - 0.0) * 49 = 49 (bottom). The horizontal line spans x=0..99.
        // Check that all visible pixels are on the same row
        let visible_rows: std::collections::HashSet<u32> = buf
            .chunks(4)
            .enumerate()
            .filter(|(_, px)| px[3] > 0)
            .map(|(i, _)| (i as u32 / 100) as u32)
            .collect();
        assert_eq!(
            visible_rows.len(),
            1,
            "single BPM should produce a single horizontal line"
        );
    }

    #[test]
    fn bpm_graph_bpm_change_has_pixels_at_change() {
        let events = vec![(0, 120.0), (1_000_000, 180.0)];
        let buf = compute_bpm_graph_pixels(&events, 100, 50);
        assert!(has_visible_pixels(&buf));
        // Should have both horizontal segments and a vertical transition
        let visible_count = count_visible_pixels(&buf);
        assert!(
            visible_count > 100,
            "BPM change should produce more pixels than a single line"
        );
    }

    #[test]
    fn hit_error_empty() {
        let buf = compute_hit_error_pixels(&[], 100, 50);
        assert_eq!(buf.len(), 100 * 50 * 4);
        assert!(!has_visible_pixels(&buf));
    }

    #[test]
    fn hit_error_symmetric_distribution() {
        // Symmetric errors: equal number early and late
        let errors: Vec<i64> = (-10..=10).map(|x| x * 1000).collect();
        let buf = compute_hit_error_pixels(&errors, 100, 50);
        assert!(has_visible_pixels(&buf));

        // Check left-right symmetry: count visible pixels in left half vs right half
        let left_count = buf
            .chunks(4)
            .enumerate()
            .filter(|(i, px)| (*i as u32 % 100) < 50 && px[3] > 0)
            .count();
        let right_count = buf
            .chunks(4)
            .enumerate()
            .filter(|(i, px)| (*i as u32 % 100) >= 50 && px[3] > 0)
            .count();
        // Allow some tolerance for rounding at center bin
        let diff = (left_count as i64 - right_count as i64).unsigned_abs();
        assert!(
            diff <= (50 * 2) as u64,
            "symmetric errors should produce roughly symmetric pixels: left={left_count}, right={right_count}"
        );
    }

    #[test]
    fn note_distribution_uniform_equal_height() {
        let lane_counts = vec![100, 100, 100, 100];
        let buf = compute_note_distribution_pixels(&lane_counts, 100, 50);
        assert!(has_visible_pixels(&buf));

        // With uniform counts, all bars should have the same height
        let bar_width = 100 / 4;
        // Check that each lane has the same number of visible pixels
        let mut lane_pixel_counts = Vec::new();
        for lane in 0..4u32 {
            let x_start = lane * bar_width;
            let x_end = x_start + bar_width;
            let count = buf
                .chunks(4)
                .enumerate()
                .filter(|(i, px)| {
                    let x = *i as u32 % 100;
                    x >= x_start && x < x_end && px[3] > 0
                })
                .count();
            lane_pixel_counts.push(count);
        }
        // All lanes should have equal pixel counts
        assert!(
            lane_pixel_counts.iter().all(|c| *c == lane_pixel_counts[0]),
            "uniform counts should produce equal-height bars: {lane_pixel_counts:?}"
        );
    }

    #[test]
    fn timing_distribution_all_pg() {
        let judge_counts = [1000, 0, 0, 0, 0];
        let buf = compute_timing_distribution_pixels(&judge_counts, 100, 50);
        assert!(has_visible_pixels(&buf));

        // All visible pixels should be cyan (PG color: 0, 255, 255)
        for px in buf.chunks(4) {
            if px[3] > 0 {
                assert_eq!(px[0], 0, "R should be 0 for PG");
                assert_eq!(px[1], 255, "G should be 255 for PG");
                assert_eq!(px[2], 255, "B should be 255 for PG");
            }
        }
    }

    #[test]
    fn timing_visualizer_empty() {
        let buf = compute_timing_visualizer_pixels(&[], 100, 50);
        // Center line is still drawn, so there should be some visible pixels
        // But the data dots should be absent — just verify buffer size
        assert_eq!(buf.len(), 100 * 50 * 4);
        assert!(!has_visible_pixels(&buf));
    }

    #[test]
    fn timing_visualizer_data_present() {
        let data = vec![(0, -5000), (500_000, 0), (1_000_000, 3000)];
        let buf = compute_timing_visualizer_pixels(&data, 100, 50);
        assert!(has_visible_pixels(&buf));
        // Should have more visible pixels than just the center line (3 dots × 9 pixels + line)
        let visible = count_visible_pixels(&buf);
        assert!(
            visible > 100,
            "should have dots plus center line: {visible} pixels"
        );
    }
}
