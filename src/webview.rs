use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};
use wry::{
    dpi::{PhysicalPosition, PhysicalSize},
    WebView,
};

#[derive(Resource)]
pub struct ScaleFactor(f64);

impl ScaleFactor {
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

impl From<f64> for ScaleFactor {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

pub fn keep_webview_fullscreen(
    mut resize_reader: EventReader<WindowResized>,
    webview: NonSendMut<WebView>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
) {
    let primary_window = primary_window_entity.single();
    for resize_event in resize_reader.read() {
        let WindowResized {
            window,
            width,
            height,
        } = resize_event;
        if *window != primary_window {
            continue;
        }

        webview
            .set_bounds(wry::Rect {
                position: wry::dpi::Position::new(PhysicalPosition::new(0, 0)),
                size: wry::dpi::Size::new(PhysicalSize::new(*width as u32, *height as u32)),
            })
            .unwrap();
    }
}
