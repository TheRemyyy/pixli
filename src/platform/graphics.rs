//! Graphics backend and presentation mode selection.

/// Native graphics backends supported by this build.
pub fn graphics_backends() -> wgpu::Backends {
    #[cfg(target_os = "linux")]
    {
        wgpu::Backends::VULKAN | wgpu::Backends::GL
    }

    #[cfg(not(target_os = "linux"))]
    {
        wgpu::Backends::PRIMARY | wgpu::Backends::GL
    }
}

/// Select a present mode that is actually supported by the created surface.
pub fn select_present_mode(
    vsync: bool,
    supported_modes: &[wgpu::PresentMode],
) -> wgpu::PresentMode {
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

#[cfg(test)]
mod tests {
    use super::select_present_mode;

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
}
