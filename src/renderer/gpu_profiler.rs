//! GPU timestamp profiler.

use std::sync::{Arc, Mutex};

const PASS_COUNT: usize = 7;
const QUERY_COUNT: u32 = (PASS_COUNT as u32) * 2;
const QUERY_BUFFER_SIZE: u64 = QUERY_COUNT as u64 * std::mem::size_of::<u64>() as u64;

#[derive(Clone, Copy)]
pub(super) enum GpuPass {
    Frame = 0,
    Depth = 1,
    Shadow = 2,
    Main = 3,
    Bloom = 4,
    Ssao = 5,
    Post = 6,
}

impl GpuPass {
    fn label(self) -> &'static str {
        match self {
            Self::Frame => "gpu",
            Self::Depth => "depth",
            Self::Shadow => "shadow",
            Self::Main => "main",
            Self::Bloom => "bloom",
            Self::Ssao => "ssao",
            Self::Post => "post",
        }
    }

    fn query_start(self) -> u32 {
        self as u32 * 2
    }
}

type MapResult = Arc<Mutex<Option<Result<(), wgpu::BufferAsyncError>>>>;

pub(super) struct GpuProfiler {
    enabled: bool,
    query_set: Option<wgpu::QuerySet>,
    resolve_buffer: Option<wgpu::Buffer>,
    readback_buffer: Option<wgpu::Buffer>,
    timestamp_period: f32,
    pending_map: Option<MapResult>,
    latest_ms: [Option<f64>; PASS_COUNT],
    frame_active: bool,
    log_enabled: bool,
    samples_since_log: u32,
    log_interval: u32,
}

impl GpuProfiler {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let required =
            wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::TIMESTAMP_QUERY_INSIDE_ENCODERS;
        let enabled = device.features().contains(required);
        if !enabled {
            return Self {
                enabled,
                query_set: None,
                resolve_buffer: None,
                readback_buffer: None,
                timestamp_period: 0.0,
                pending_map: None,
                latest_ms: [None; PASS_COUNT],
                frame_active: false,
                log_enabled: false,
                samples_since_log: 0,
                log_interval: 120,
            };
        }

        let query_set = device.create_query_set(&wgpu::QuerySetDescriptor {
            label: Some("Pixli GPU Frame Queries"),
            ty: wgpu::QueryType::Timestamp,
            count: QUERY_COUNT,
        });
        let resolve_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pixli GPU Timestamp Resolve"),
            size: QUERY_BUFFER_SIZE,
            usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pixli GPU Timestamp Readback"),
            size: QUERY_BUFFER_SIZE,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            enabled,
            query_set: Some(query_set),
            resolve_buffer: Some(resolve_buffer),
            readback_buffer: Some(readback_buffer),
            timestamp_period: queue.get_timestamp_period(),
            pending_map: None,
            latest_ms: [None; PASS_COUNT],
            frame_active: false,
            log_enabled: std::env::var("PIXLI_PROFILE").is_ok(),
            samples_since_log: 0,
            log_interval: std::env::var("PIXLI_PROFILE_INTERVAL")
                .ok()
                .and_then(|value| value.parse::<u32>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(120),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn latest_summary(&self) -> Option<String> {
        let mut parts = Vec::new();
        for pass in [
            GpuPass::Frame,
            GpuPass::Depth,
            GpuPass::Shadow,
            GpuPass::Main,
            GpuPass::Bloom,
            GpuPass::Ssao,
            GpuPass::Post,
        ] {
            if let Some(ms) = self.latest_ms[pass as usize] {
                parts.push(format!("{}={ms:.3}ms", pass.label()));
            }
        }
        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }

    pub fn begin_frame(&mut self, encoder: &mut wgpu::CommandEncoder) {
        self.frame_active = false;
        if self.pending_map.is_some() {
            return;
        }
        self.write_start(encoder, GpuPass::Frame);
        self.frame_active = true;
    }

    pub fn end_frame(&self, encoder: &mut wgpu::CommandEncoder) {
        if !self.frame_active {
            return;
        }
        let (Some(query_set), Some(resolve_buffer), Some(readback_buffer)) =
            (&self.query_set, &self.resolve_buffer, &self.readback_buffer)
        else {
            return;
        };
        encoder.write_timestamp(query_set, GpuPass::Frame.query_start() + 1);
        encoder.resolve_query_set(query_set, 0..QUERY_COUNT, resolve_buffer, 0);
        encoder.copy_buffer_to_buffer(resolve_buffer, 0, readback_buffer, 0, QUERY_BUFFER_SIZE);
    }

    pub fn begin_pass(&self, encoder: &mut wgpu::CommandEncoder, pass: GpuPass) {
        if self.frame_active {
            self.write_start(encoder, pass);
        }
    }

    pub fn end_pass(&self, encoder: &mut wgpu::CommandEncoder, pass: GpuPass) {
        if self.frame_active {
            self.write_end(encoder, pass);
        }
    }

    fn write_start(&self, encoder: &mut wgpu::CommandEncoder, pass: GpuPass) {
        if let Some(query_set) = &self.query_set {
            encoder.write_timestamp(query_set, pass.query_start());
        }
    }

    fn write_end(&self, encoder: &mut wgpu::CommandEncoder, pass: GpuPass) {
        if let Some(query_set) = &self.query_set {
            encoder.write_timestamp(query_set, pass.query_start() + 1);
        }
    }

    pub fn begin_readback(&mut self) {
        if !self.enabled || !self.frame_active || self.pending_map.is_some() {
            return;
        }
        let Some(buffer) = &self.readback_buffer else {
            return;
        };
        let map_result = Arc::new(Mutex::new(None));
        let callback_result = map_result.clone();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Ok(mut slot) = callback_result.lock() {
                    *slot = Some(result);
                }
            });
        self.pending_map = Some(map_result);
        self.frame_active = false;
    }

    pub fn collect(&mut self, device: &wgpu::Device) {
        if self.pending_map.is_none() {
            return;
        }
        let _ = device.poll(wgpu::Maintain::Poll);
        let Some(map_result) = &self.pending_map else {
            return;
        };
        let Some(result) = map_result.lock().ok().and_then(|mut slot| slot.take()) else {
            return;
        };
        self.pending_map = None;
        if result.is_err() {
            return;
        }
        if let Some(buffer) = &self.readback_buffer {
            let data = buffer.slice(..).get_mapped_range();
            let timestamps: &[u64] = bytemuck::cast_slice(&data);
            if timestamps.len() >= QUERY_COUNT as usize {
                for pass_index in 0..PASS_COUNT {
                    let start_index = pass_index * 2;
                    let end_index = start_index + 1;
                    if timestamps[end_index] >= timestamps[start_index] {
                        let elapsed_ticks = timestamps[end_index] - timestamps[start_index];
                        self.latest_ms[pass_index] = Some(
                            elapsed_ticks as f64 * f64::from(self.timestamp_period) / 1_000_000.0,
                        );
                    }
                }
                self.samples_since_log += 1;
                if self.log_enabled && self.samples_since_log >= self.log_interval {
                    if let Some(summary) = self.latest_summary() {
                        log::info!("GPU profiler: {summary}");
                    }
                    self.samples_since_log = 0;
                }
            }
            drop(data);
            buffer.unmap();
        }
    }
}
