use bevy::math::IVec2;
use bevy::utils::HashMap;

use bevy::window::{RawHandleWrapper, Window};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window as WinitWindow, WindowId};

#[derive(Debug, Default)]
pub struct WinitWindows {
    pub windows: Vec<WinitWindow>,
    // Winit is safe to call only from main thread
    _not_send_sync: core::marker::PhantomData<*const ()>,
}

impl WinitWindows {
    pub fn create_window(
        &mut self,
        event_loop: &EventLoopWindowTarget<()>,
        window_id: WindowId,
        window: &Window,
    ) {
    }
}
