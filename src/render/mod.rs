//! Renderer backend. Vulkan (via ash) is the only planned backend.
//! Phase 0: stub that proves the window lifecycle works.
//! Phase 1 will create the Vulkan instance, Android surface, swapchain and clear the screen.

use android_activity::NativeWindow;
use log::info;

pub struct Renderer {
    frame_index: u64,
}

impl Renderer {
    pub fn new(_window: &NativeWindow) -> Result<Self, String> {
        info!("renderer: window ready (Vulkan backend not implemented yet)");
        Ok(Self { frame_index: 0 })
    }

    pub fn frame(&mut self) {
        self.frame_index += 1;
    }
}
