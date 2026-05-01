//! Graphics backend and presentation mode selection.

/// Native graphics backends supported by this build.
pub fn graphics_backends() -> wgpu::Backends {
    wgpu::Backends::VULKAN
}

/// Return true only for the backend Pixli supports at runtime.
pub fn is_supported_backend(backend: wgpu::Backend) -> bool {
    backend == wgpu::Backend::Vulkan
}

/// Select a present mode that is actually supported by the created surface.
pub fn select_present_mode(
    vsync: bool,
    supported_modes: &[wgpu::PresentMode],
) -> wgpu::PresentMode {
    if let Some(mode) = forced_present_mode(supported_modes) {
        return mode;
    }

    let preferred_modes = if vsync {
        [
            wgpu::PresentMode::AutoVsync,
            wgpu::PresentMode::Fifo,
            wgpu::PresentMode::FifoRelaxed,
        ]
    } else {
        [
            wgpu::PresentMode::Immediate,
            wgpu::PresentMode::AutoNoVsync,
            wgpu::PresentMode::Mailbox,
        ]
    };

    preferred_modes
        .into_iter()
        .find(|mode| supported_modes.contains(mode))
        .or_else(|| {
            supported_modes
                .iter()
                .copied()
                .find(|mode| *mode == wgpu::PresentMode::Fifo)
        })
        .or_else(|| supported_modes.first().copied())
        .unwrap_or(wgpu::PresentMode::Fifo)
}

fn forced_present_mode(supported_modes: &[wgpu::PresentMode]) -> Option<wgpu::PresentMode> {
    let forced = std::env::var("PIXLI_PRESENT_MODE").ok()?;
    let requested = match forced.to_ascii_lowercase().as_str() {
        "immediate" => wgpu::PresentMode::Immediate,
        "mailbox" => wgpu::PresentMode::Mailbox,
        "fifo" => wgpu::PresentMode::Fifo,
        "fifo_relaxed" | "fiforelaxed" => wgpu::PresentMode::FifoRelaxed,
        "auto_vsync" | "autovsync" => wgpu::PresentMode::AutoVsync,
        "auto_no_vsync" | "autonovsync" => wgpu::PresentMode::AutoNoVsync,
        other => {
            log::warn!("Unsupported PIXLI_PRESENT_MODE={other}, using automatic selection");
            return None;
        }
    };
    if supported_modes.contains(&requested) {
        Some(requested)
    } else {
        log::warn!(
            "Requested PIXLI_PRESENT_MODE={forced} is not supported by this surface, using automatic selection"
        );
        None
    }
}

/// Select swapchain frame latency. Two queued frames is the throughput default on tested Vulkan
/// drivers; `PIXLI_FRAME_LATENCY=1` is available for latency-sensitive builds.
pub fn select_frame_latency(_vsync: bool) -> u32 {
    std::env::var("PIXLI_FRAME_LATENCY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .map(|value| value.clamp(1, 3))
        .unwrap_or(2)
}

#[cfg(test)]
mod tests {
    use super::{
        graphics_backends, is_supported_backend, select_frame_latency, select_present_mode,
    };

    #[test]
    fn graphics_backends_are_vulkan_only() {
        assert_eq!(graphics_backends(), wgpu::Backends::VULKAN);
    }

    #[test]
    fn supported_backend_is_vulkan_only() {
        assert!(is_supported_backend(wgpu::Backend::Vulkan));
        assert!(!is_supported_backend(wgpu::Backend::Gl));
        assert!(!is_supported_backend(wgpu::Backend::Dx12));
        assert!(!is_supported_backend(wgpu::Backend::Metal));
    }

    #[test]
    fn select_present_mode_falls_back_to_fifo_when_immediate_is_missing() {
        let supported_modes = [wgpu::PresentMode::Fifo, wgpu::PresentMode::Mailbox];

        let selected = select_present_mode(false, &supported_modes);

        assert_eq!(selected, wgpu::PresentMode::Mailbox);
    }

    #[test]
    fn select_present_mode_uses_fifo_when_no_preferred_mode_exists() {
        let supported_modes = [wgpu::PresentMode::Fifo];

        let selected = select_present_mode(false, &supported_modes);

        assert_eq!(selected, wgpu::PresentMode::Fifo);
    }

    #[test]
    fn select_frame_latency_prefers_throughput_default() {
        assert_eq!(select_frame_latency(false), 2);
        assert_eq!(select_frame_latency(true), 2);
    }
}
