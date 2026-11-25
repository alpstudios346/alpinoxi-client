//! Alpinoxi Performance: the measurements and the advice.
//!
//! The performance mod itself is a Minecraft mod, but its *judgement* lives here
//! so it can be tested without the game:
//!
//! * a frame-time histogram with honest percentiles (no averaged FPS tricks),
//! * a diagnosis that says what is actually slow: CPU-bound frames, GPU-bound
//!   frames, or memory pressure in the heap,
//! * a render-distance recommendation built from the measured headroom,
//! * an optional 1% low tracker that keeps a bounded history.
//!
//! Nothing here draws or patches anything: it measures and it recommends. That
//! separation is what makes the numbers trustworthy.

use std::collections::VecDeque;

/// A fixed-size ring of frame times, in milliseconds.
#[derive(Debug, Clone, PartialEq)]
pub struct FrameHistogram {
    samples: VecDeque<f32>,
    capacity: usize,
}

impl FrameHistogram {
    pub fn new(capacity: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(capacity.max(8)),
            capacity: capacity.max(8),
        }
    }

    pub fn record(&mut self, frame_ms: f32) {
        if !frame_ms.is_finite() || frame_ms <= 0.0 {
            return;
        }
        if self.samples.len() == self.capacity {
            self.samples.pop_front();
        }
        self.samples.push_back(frame_ms);
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Average FPS over the window.
    pub fn average_fps(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let total: f32 = self.samples.iter().sum();
        let mean = total / self.samples.len() as f32;
        if mean > 0.0 {
            1_000.0 / mean
        } else {
            0.0
        }
    }

    /// `percentile` of frame times in milliseconds, 0..=100.
    ///
    /// Uses nearest-rank on a sorted copy. For 0.1% low FPS the caller asks for
    /// the 99.9th percentile of frame time and inverts it.
    pub fn frame_time_percentile(&self, percentile: f32) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f32> = self.samples.iter().copied().collect();
        sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
        let rank = ((percentile.clamp(0.0, 100.0) / 100.0) * (sorted.len() - 1) as f32).round() as usize;
        sorted[rank.min(sorted.len() - 1)]
    }

    /// 1% low FPS: the FPS a player actually feels in the worst moments.
    pub fn one_percent_low_fps(&self) -> f32 {
        let worst = self.frame_time_percentile(99.0);
        if worst > 0.0 {
            1_000.0 / worst
        } else {
            0.0
        }
    }

    /// 0.1% low FPS, for stutter hunting.
    pub fn point_one_percent_low_fps(&self) -> f32 {
        let worst = self.frame_time_percentile(99.9);
        if worst > 0.0 {
            1_000.0 / worst
        } else {
            0.0
        }
    }

    pub fn max_frame_ms(&self) -> f32 {
        self.samples.iter().copied().fold(0.0, f32::max)
    }
}

impl Default for FrameHistogram {
    fn default() -> Self {
        Self::new(600)
    }
}

/// What is holding the frame back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bottleneck {
    /// Long frames, low GPU load: the game logic or chunk meshing.
    Cpu,
    /// Long frames, high GPU load: resolution, shaders or chunk count.
    Gpu,
    /// Heap near its ceiling, garbage collection pauses in the frame time.
    Memory,
    /// The measurement window is too short to say anything.
    Unknown,
}

impl Bottleneck {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Cpu => "CPU-bound",
            Self::Gpu => "GPU-bound",
            Self::Memory => "Memory pressure",
            Self::Unknown => "Not enough samples",
        }
    }
}

/// Inputs for a diagnosis. The mod fills these from its own counters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameStats {
    pub average_fps: f32,
    pub one_percent_low_fps: f32,
    pub max_frame_ms: f32,
    pub gpu_load_percent: u8,
    pub heap_used_mib: u32,
    pub heap_max_mib: u32,
    pub samples: usize,
}

impl FrameStats {
    pub fn from_histogram(
        histogram: &FrameHistogram,
        gpu_load_percent: u8,
        heap_used_mib: u32,
        heap_max_mib: u32,
    ) -> Self {
        Self {
            average_fps: histogram.average_fps(),
            one_percent_low_fps: histogram.one_percent_low_fps(),
            max_frame_ms: histogram.max_frame_ms(),
            gpu_load_percent,
            heap_used_mib,
            heap_max_mib,
            samples: histogram.len(),
        }
    }
}

/// A recommendation the mod shows in its own settings screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Advice {
    pub bottleneck: Bottleneck,
    pub headline: String,
    pub actions: Vec<String>,
    /// Suggested render distance, or `None` when the current one is fine.
    pub render_distance: Option<u8>,
}

/// Turns numbers into one clear next step instead of a wall of sliders.
pub fn diagnose(stats: FrameStats, current_render_distance: u8) -> Advice {
    if stats.samples < 60 {
        return Advice {
            bottleneck: Bottleneck::Unknown,
            headline: "Play for a few seconds so Alpinoxi can measure.".to_owned(),
            actions: Vec::new(),
            render_distance: None,
        };
    }

    let heap_ratio = if stats.heap_max_mib == 0 {
        0.0
    } else {
        stats.heap_used_mib as f32 / stats.heap_max_mib as f32
    };

    if heap_ratio >= 0.92 {
        return Advice {
            bottleneck: Bottleneck::Memory,
            headline: "The heap is almost full, so garbage collection is eating frames.".to_owned(),
            actions: vec![
                "Raise the heap by one step in the launcher, but not past half of system RAM.".to_owned(),
                "Install FerriteCore: it often saves hundreds of megabytes on its own.".to_owned(),
            ],
            render_distance: None,
        };
    }

    if stats.gpu_load_percent >= 95 && stats.average_fps < 144.0 {
        let step = current_render_distance.saturating_sub(2).max(4);
        return Advice {
            bottleneck: Bottleneck::Gpu,
            headline: "The GPU is the limit: it is busy while the frame time climbs.".to_owned(),
            actions: vec![
                "Lower the render distance by two chunks first, it costs the most pixels.".to_owned(),
                "Turn off shaders or drop their shadow resolution.".to_owned(),
            ],
            render_distance: Some(step),
        };
    }

    if stats.one_percent_low_fps < stats.average_fps * 0.55 {
        return Advice {
            bottleneck: Bottleneck::Cpu,
            headline: "Average FPS is fine but the lows are not: this is stutter, not low speed.".to_owned(),
            actions: vec![
                "Make sure Sodium and Lithium are both installed and up to date.".to_owned(),
                "Add Entity Culling and ImmediatelyFast: they cut per-frame CPU work.".to_owned(),
                "Cap the frame rate to your monitor's refresh rate to flatten frame times.".to_owned(),
            ],
            render_distance: None,
        };
    }

    Advice {
        bottleneck: Bottleneck::Cpu,
        headline: "Frames are steady. If you want more, the ceiling is the CPU, not the GPU.".to_owned(),
        actions: vec![
            "Keep the render distance: dropping it would not help while the GPU is idle.".to_owned(),
            "Turn down simulation distance if redstone-heavy areas stutter.".to_owned(),
        ],
        render_distance: None,
    }
}

/// A minimal adaptive render distance: it only moves one chunk at a time and
/// only when the measured frame time stays on one side of the target, so the
/// view never pops back and forth.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGovernor {
    pub current: u8,
    pub min: u8,
    pub max: u8,
    target_fps: u32,
    good_frames: u32,
    bad_frames: u32,
}

impl RenderGovernor {
    pub fn new(current: u8, min: u8, max: u8, target_fps: u32) -> Self {
        Self {
            current: current.clamp(min, max),
            min,
            max,
            target_fps: target_fps.max(30),
            good_frames: 0,
            bad_frames: 0,
        }
    }

    /// Feeds one second of average FPS. Returns the new distance if it changed.
    pub fn update(&mut self, average_fps: f32) -> Option<u8> {
        if average_fps <= 0.0 {
            return None;
        }
        let target = self.target_fps as f32;
        if average_fps >= target * 1.12 {
            self.good_frames += 1;
            self.bad_frames = 0;
        } else if average_fps < target * 0.92 {
            self.bad_frames += 1;
            self.good_frames = 0;
        } else {
            self.good_frames = 0;
            self.bad_frames = 0;
            return None;
        }

        // Five steady seconds before a change: slow enough to never oscillate.
        if self.good_frames >= 5 && self.current < self.max {
            self.current += 1;
            self.good_frames = 0;
            return Some(self.current);
        }
        if self.bad_frames >= 3 && self.current > self.min {
            self.current -= 1;
            self.bad_frames = 0;
            return Some(self.current);
        }
        None
    }

    /// What the HUD prints next to the FPS counter.
    pub fn hud_label(&self) -> String {
        format!("{} chunks · target {} FPS", self.current, self.target_fps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn histogram_with(samples: &[f32]) -> FrameHistogram {
        let mut histogram = FrameHistogram::new(600);
        for sample in samples {
            histogram.record(*sample);
        }
        histogram
    }

    #[test]
    fn ring_buffer_keeps_the_newest_samples() {
        let mut histogram = FrameHistogram::new(10);
        for index in 0..25 {
            histogram.record(index as f32);
        }
        assert_eq!(histogram.len(), 10);
        assert_eq!(histogram.max_frame_ms(), 24.0);
    }

    #[test]
    fn rejects_nonsense_samples() {
        let mut histogram = FrameHistogram::new(10);
        histogram.record(f32::NAN);
        histogram.record(-5.0);
        histogram.record(0.0);
        assert!(histogram.is_empty());
    }

    #[test]
    fn percentiles_track_the_worst_frames() {
        let mut samples: Vec<f32> = vec![8.0; 99];
        samples.push(40.0); // one 25 FPS frame
        let histogram = histogram_with(&samples);
        assert!(histogram.average_fps() > 100.0);
        assert!((histogram.one_percent_low_fps() - 25.0).abs() < 0.01);
        assert!(histogram.point_one_percent_low_fps() <= histogram.one_percent_low_fps());
    }

    #[test]
    fn short_windows_are_reported_as_unknown() {
        let histogram = histogram_with(&[16.0; 10]);
        let advice = diagnose(
            FrameStats::from_histogram(&histogram, 40, 1_000, 4_096),
            12,
        );
        assert_eq!(advice.bottleneck, Bottleneck::Unknown);
    }

    #[test]
    fn full_heap_is_diagnosed_first() {
        let histogram = histogram_with(&[16.0; 120]);
        let advice = diagnose(
            FrameStats::from_histogram(&histogram, 40, 3_950, 4_096),
            12,
        );
        assert_eq!(advice.bottleneck, Bottleneck::Memory);
        assert!(advice.actions.iter().any(|action| action.contains("FerriteCore")));
    }

    #[test]
    fn busy_gpu_lowers_render_distance_by_two() {
        let histogram = histogram_with(&[33.0; 120]);
        let advice = diagnose(
            FrameStats::from_histogram(&histogram, 99, 1_500, 4_096),
            12,
        );
        assert_eq!(advice.bottleneck, Bottleneck::Gpu);
        assert_eq!(advice.render_distance, Some(10));
    }

    #[test]
    fn render_distance_never_goes_below_four() {
        let histogram = histogram_with(&[33.0; 120]);
        let advice = diagnose(FrameStats::from_histogram(&histogram, 99, 1_500, 4_096), 5);
        assert_eq!(advice.render_distance, Some(4));
    }

    #[test]
    fn stutter_is_called_stutter() {
        let mut samples = vec![10.0; 118];
        samples.extend_from_slice(&[80.0, 80.0]);
        let histogram = histogram_with(&samples);
        let advice = diagnose(
            FrameStats::from_histogram(&histogram, 60, 2_000, 4_096),
            12,
        );
        assert_eq!(advice.bottleneck, Bottleneck::Cpu);
        assert!(advice.headline.contains("stutter"));
    }

    #[test]
    fn governor_climbs_slowly_and_falls_quickly() {
        let mut governor = RenderGovernor::new(10, 4, 16, 60);
        assert_eq!(governor.update(120.0), None);
        for _ in 0..4 {
            governor.update(120.0);
        }
        assert_eq!(governor.update(120.0), Some(11));
        assert_eq!(governor.update(30.0), None);
        governor.update(30.0);
        assert_eq!(governor.update(30.0), Some(10));
        assert!(governor.hud_label().contains("target 60 FPS"));
    }

    #[test]
    fn governor_respects_its_bounds() {
        let mut governor = RenderGovernor::new(16, 4, 16, 60);
        for _ in 0..20 {
            governor.update(200.0);
        }
        assert_eq!(governor.current, 16);
        let mut governor = RenderGovernor::new(4, 4, 16, 60);
        for _ in 0..20 {
            governor.update(10.0);
        }
        assert_eq!(governor.current, 4);
    }
}
