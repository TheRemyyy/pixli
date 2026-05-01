//! Lightweight frame profiler for CPU-side engine timing.

use std::time::Duration;

const DEFAULT_REPORT_INTERVAL_FRAMES: u32 = 120;

/// Per-frame CPU timings collected by the app loop.
#[derive(Debug, Clone, Copy, Default)]
pub struct FrameProfile {
    pub physics: Duration,
    pub systems: Duration,
    pub acquire_surface: Duration,
    pub render: Duration,
    pub present: Duration,
    pub total: Duration,
    pub frame_interval: Duration,
    pub present_mode: Option<wgpu::PresentMode>,
    pub frame_latency: u32,
}

#[derive(Debug, Clone, Copy, Default)]
struct DurationStats {
    total: Duration,
    max: Duration,
}

impl DurationStats {
    fn record(&mut self, duration: Duration) {
        self.total += duration;
        self.max = self.max.max(duration);
    }

    fn average_ms(self, samples: u32) -> f64 {
        if samples == 0 {
            return 0.0;
        }
        self.total.as_secs_f64() * 1000.0 / f64::from(samples)
    }

    fn max_ms(self) -> f64 {
        self.max.as_secs_f64() * 1000.0
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct FrameStats {
    physics: DurationStats,
    systems: DurationStats,
    acquire_surface: DurationStats,
    render: DurationStats,
    present: DurationStats,
    total: DurationStats,
    frame_interval: DurationStats,
    best_frame_interval: Option<Duration>,
}

impl FrameStats {
    fn record(&mut self, profile: FrameProfile) {
        self.physics.record(profile.physics);
        self.systems.record(profile.systems);
        self.acquire_surface.record(profile.acquire_surface);
        self.render.record(profile.render);
        self.present.record(profile.present);
        self.total.record(profile.total);
        self.frame_interval.record(profile.frame_interval);
        self.best_frame_interval = Some(
            self.best_frame_interval
                .map(|best| best.min(profile.frame_interval))
                .unwrap_or(profile.frame_interval),
        );
    }
}

/// Runtime profiler controlled by `PIXLI_PROFILE=1`.
pub struct Profiler {
    enabled: bool,
    report_interval_frames: u32,
    frames_since_report: u32,
    stats: FrameStats,
    latest_summary: String,
    present_mode: Option<wgpu::PresentMode>,
    frame_latency: u32,
}

impl Profiler {
    pub fn from_env() -> Self {
        let enabled = std::env::var("PIXLI_PROFILE")
            .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "on" | "ON"))
            .unwrap_or(false);
        let report_interval_frames = std::env::var("PIXLI_PROFILE_INTERVAL")
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_REPORT_INTERVAL_FRAMES);

        Self {
            enabled,
            report_interval_frames,
            frames_since_report: 0,
            stats: FrameStats::default(),
            latest_summary: String::new(),
            present_mode: None,
            frame_latency: 0,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn latest_summary(&self) -> &str {
        &self.latest_summary
    }

    pub fn record(&mut self, profile: FrameProfile) {
        if !self.enabled {
            return;
        }

        self.stats.record(profile);
        self.present_mode = profile.present_mode;
        self.frame_latency = profile.frame_latency;
        self.frames_since_report += 1;

        if self.frames_since_report >= self.report_interval_frames {
            self.latest_summary = format_summary(
                self.stats,
                self.frames_since_report,
                self.present_mode,
                self.frame_latency,
            );
            log::info!("Profiler: {}", self.latest_summary);
            self.frames_since_report = 0;
            self.stats = FrameStats::default();
        }
    }
}

fn format_summary(
    stats: FrameStats,
    samples: u32,
    present_mode: Option<wgpu::PresentMode>,
    frame_latency: u32,
) -> String {
    let avg_frame_ms = stats.frame_interval.average_ms(samples);
    let avg_fps = if avg_frame_ms > 0.0 {
        1000.0 / avg_frame_ms
    } else {
        0.0
    };
    let best_fps = stats
        .best_frame_interval
        .map(|duration| {
            let ms = duration.as_secs_f64() * 1000.0;
            if ms > 0.0 {
                1000.0 / ms
            } else {
                0.0
            }
        })
        .unwrap_or(0.0);
    format!(
        "avg ms total={:.2} frame={:.2} physics={:.2} systems={:.2} acquire={:.2} render={:.2} present={:.2}; max total={:.2} acquire={:.2} render={:.2} present={:.2}; fps avg={:.0} best={:.0}; present={:?} latency={}",
        stats.total.average_ms(samples),
        avg_frame_ms,
        stats.physics.average_ms(samples),
        stats.systems.average_ms(samples),
        stats.acquire_surface.average_ms(samples),
        stats.render.average_ms(samples),
        stats.present.average_ms(samples),
        stats.total.max_ms(),
        stats.acquire_surface.max_ms(),
        stats.render.max_ms(),
        stats.present.max_ms(),
        avg_fps,
        best_fps,
        present_mode,
        frame_latency,
    )
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{format_summary, FrameProfile, FrameStats};

    #[test]
    fn format_summary_reports_average_and_max_timings() {
        let mut stats = FrameStats::default();
        stats.record(FrameProfile {
            physics: Duration::from_millis(1),
            systems: Duration::from_millis(2),
            acquire_surface: Duration::from_millis(3),
            render: Duration::from_millis(4),
            present: Duration::from_millis(5),
            total: Duration::from_millis(15),
            frame_interval: Duration::from_millis(10),
            present_mode: Some(wgpu::PresentMode::Immediate),
            frame_latency: 1,
        });
        stats.record(FrameProfile {
            physics: Duration::from_millis(3),
            systems: Duration::from_millis(4),
            acquire_surface: Duration::from_millis(5),
            render: Duration::from_millis(6),
            present: Duration::from_millis(7),
            total: Duration::from_millis(25),
            frame_interval: Duration::from_millis(20),
            present_mode: Some(wgpu::PresentMode::Immediate),
            frame_latency: 1,
        });

        let summary = format_summary(stats, 2, Some(wgpu::PresentMode::Immediate), 1);

        assert!(summary.contains("total=20.00"));
        assert!(summary.contains("physics=2.00"));
        assert!(summary.contains("render=5.00"));
        assert!(summary.contains("max total=25.00"));
        assert!(summary.contains("present=Some(Immediate)"));
    }
}
