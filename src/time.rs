//! Time management: delta time, FPS, timers.

use std::collections::VecDeque;
use std::time::Instant;

/// Time state.
pub struct Time {
    delta: f32,
    elapsed: f32,
    frame_count: u64,
    start_time: Instant,
    #[allow(dead_code)]
    last_frame: Instant,

    // FPS calculation.
    fps: f32,
    frame_times: VecDeque<f32>,
    fps_update_timer: f32,

    // Time scale for slow motion effects.
    time_scale: f32,

    // Fixed timestep accumulator.
    fixed_delta: f32,
    fixed_accumulator: f32,
}

impl Time {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            delta: 0.0,
            elapsed: 0.0,
            frame_count: 0,
            start_time: now,
            last_frame: now,
            fps: 0.0,
            frame_times: VecDeque::with_capacity(120),
            fps_update_timer: 0.0,
            time_scale: 1.0,
            fixed_delta: 1.0 / 60.0, // 60 FPS fixed update.
            fixed_accumulator: 0.0,
        }
    }

    /// Update time (called each frame).
    pub fn update(&mut self, delta: f32) {
        self.delta = delta;
        self.elapsed += delta;
        self.frame_count += 1;

        // FPS calculation.
        self.frame_times.push_back(delta);
        if self.frame_times.len() > 100 {
            self.frame_times.pop_front();
        }

        self.fps_update_timer += delta;
        if self.fps_update_timer >= 0.25 {
            self.fps_update_timer = 0.0;
            let avg: f32 = self.frame_times.iter().sum::<f32>() / self.frame_times.len() as f32;
            self.fps = 1.0 / avg;
        }

        // Fixed timestep accumulator.
        self.fixed_accumulator += delta * self.time_scale;
    }

    /// Get delta time (seconds since last frame).
    pub fn delta(&self) -> f32 {
        self.delta * self.time_scale
    }

    /// Get raw delta time (without time scale).
    pub fn delta_raw(&self) -> f32 {
        self.delta
    }

    /// Get total elapsed time since start.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    /// Get total elapsed time as Duration.
    pub fn elapsed_duration(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    /// Get current frame count.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get current FPS.
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Get FPS as integer (for display).
    pub fn fps_int(&self) -> u32 {
        self.fps as u32
    }

    /// Get time scale.
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Set time scale (1.0 normal, 0.5 half speed, 2.0 double speed).
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(0.0);
    }

    /// Get fixed delta time (for physics).
    pub fn fixed_delta(&self) -> f32 {
        self.fixed_delta
    }

    /// Set fixed delta time.
    pub fn set_fixed_delta(&mut self, delta: f32) {
        self.fixed_delta = delta.max(0.001);
    }

    /// Check if a fixed update should run (returns number of fixed steps).
    pub fn fixed_steps(&mut self) -> u32 {
        let mut steps = 0;
        while self.fixed_accumulator >= self.fixed_delta {
            self.fixed_accumulator -= self.fixed_delta;
            steps += 1;
            // Prevent spiral of death.
            if steps > 10 {
                self.fixed_accumulator = 0.0;
                break;
            }
        }
        steps
    }

    /// Get interpolation alpha for rendering between fixed updates.
    pub fn fixed_alpha(&self) -> f32 {
        self.fixed_accumulator / self.fixed_delta
    }

    /// Pause time (set scale to 0).
    pub fn pause(&mut self) {
        self.time_scale = 0.0;
    }

    /// Resume time (set scale to 1).
    pub fn resume(&mut self) {
        self.time_scale = 1.0;
    }

    /// Check if time is paused.
    pub fn is_paused(&self) -> bool {
        self.time_scale == 0.0
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timer.
pub struct Timer {
    duration: f32,
    elapsed: f32,
    repeating: bool,
    finished: bool,
}

impl Timer {
    /// Create a one shot timer.
    pub fn once(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: false,
            finished: false,
        }
    }

    /// Create a repeating timer.
    pub fn repeating(duration: f32) -> Self {
        Self {
            duration,
            elapsed: 0.0,
            repeating: true,
            finished: false,
        }
    }

    /// Update timer; returns true if timer just finished.
    pub fn tick(&mut self, delta: f32) -> bool {
        if self.finished && !self.repeating {
            return false;
        }

        self.elapsed += delta;

        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.finished = true;
            }
            return true;
        }

        false
    }

    /// Check if timer is finished.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Get progress (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.duration).min(1.0)
    }

    /// Get remaining time.
    pub fn remaining(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }

    /// Reset timer.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }

    /// Set duration.
    pub fn set_duration(&mut self, duration: f32) {
        self.duration = duration;
    }
}

/// Stopwatch for measuring time.
pub struct Stopwatch {
    start: Option<Instant>,
    accumulated: std::time::Duration,
    running: bool,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start: None,
            accumulated: std::time::Duration::ZERO,
            running: false,
        }
    }

    /// Start the stopwatch.
    pub fn start(&mut self) {
        if !self.running {
            self.start = Some(Instant::now());
            self.running = true;
        }
    }

    /// Stop the stopwatch.
    pub fn stop(&mut self) {
        if self.running {
            if let Some(start) = self.start {
                self.accumulated += start.elapsed();
            }
            self.running = false;
        }
    }

    /// Reset the stopwatch.
    pub fn reset(&mut self) {
        self.start = None;
        self.accumulated = std::time::Duration::ZERO;
        self.running = false;
    }

    /// Get elapsed time in seconds.
    pub fn elapsed(&self) -> f32 {
        let mut total = self.accumulated;
        if self.running {
            if let Some(start) = self.start {
                total += start.elapsed();
            }
        }
        total.as_secs_f32()
    }

    /// Check if running.
    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}
