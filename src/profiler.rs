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
}

impl FrameStats {
    fn record(&mut self, profile: FrameProfile) {
        self.physics.record(profile.physics);
        self.systems.record(profile.systems);
        self.acquire_surface.record(profile.acquire_surface);
        self.render.record(profile.render);
        self.present.record(profile.present);
        self.total.record(profile.total);
    }
}

/// Runtime profiler controlled by `PIXLI_PROFILE=1`.
pub struct Profiler {
    enabled: bool,
    report_interval_frames: u32,
    frames_since_report: u32,
    stats: FrameStats,
    latest_summary: String,
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
        self.frames_since_report += 1;

        if self.frames_since_report >= self.report_interval_frames {
            self.latest_summary = format_summary(self.stats, self.frames_since_report);
            log::info!("Profiler: {}", self.latest_summary);
            self.frames_since_report = 0;
            self.stats = FrameStats::default();
        }
    }
}

fn format_summary(stats: FrameStats, samples: u32) -> String {
    format!(
        "avg ms total={:.2} physics={:.2} systems={:.2} acquire={:.2} render={:.2} present={:.2}; max total={:.2} render={:.2}",
        stats.total.average_ms(samples),
        stats.physics.average_ms(samples),
        stats.systems.average_ms(samples),
        stats.acquire_surface.average_ms(samples),
        stats.render.average_ms(samples),
        stats.present.average_ms(samples),
        stats.total.max_ms(),
        stats.render.max_ms(),
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
        });
        stats.record(FrameProfile {
            physics: Duration::from_millis(3),
            systems: Duration::from_millis(4),
            acquire_surface: Duration::from_millis(5),
            render: Duration::from_millis(6),
            present: Duration::from_millis(7),
            total: Duration::from_millis(25),
        });

        let summary = format_summary(stats, 2);

        assert!(summary.contains("total=20.00"));
        assert!(summary.contains("physics=2.00"));
        assert!(summary.contains("render=5.00"));
        assert!(summary.contains("max total=25.00"));
    }
}
