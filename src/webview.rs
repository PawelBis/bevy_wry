use bevy::{
    prelude::*,
    utils::HashMap,
    window::{PrimaryWindow, WindowResized},
};
use winit::dpi::LogicalSize;
use wry::WebView;

use crate::communication::ui::Bounds;

pub struct WebviewWrapper {
    pub webview: WebView,
    pub bounds: Bounds,
}

#[derive(Default)]
pub struct WebViews {
    webviews: HashMap<String, WebView>,
    bounds: HashMap<String, Bounds>,
}

impl WebViews {
    pub fn insert(&mut self, name: String, webview: WebView, bounds: Bounds) {
        if self.webviews.contains_key(&name) {
            warn!("Tried to re-insert WebView {name}");
            return;
        }

        self.webviews.insert(name.clone(), webview);
        self.bounds.insert(name, bounds);
    }

    pub fn get_webview(&self, name: &String) -> Option<&WebView> {
        self.webviews.get(name)
    }

    pub fn get_bounds(&self, name: &String) -> Option<&Bounds> {
        self.bounds.get(name)
    }

    pub fn get_webview_with_bounds_mut(
        &mut self,
        name: &String,
    ) -> Option<(&mut WebView, &mut Bounds)> {
        self.webviews.get_mut(name).zip(self.bounds.get_mut(name))
    }

    pub fn get_all_webviews(&self) -> Vec<&WebView> {
        self.webviews.values().collect()
    }

    pub fn get_all_webviews_with_bounds(&self) -> Vec<(&WebView, &Bounds)> {
        self.webviews.values().zip(self.bounds.values()).collect()
    }
}

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

pub fn keep_webviews_in_bounds(
    mut resize_reader: EventReader<WindowResized>,
    webviews: NonSendMut<WebViews>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
    scale_factor: Option<Res<ScaleFactor>>,
) {
    if scale_factor.is_none() {
        return;
    }

    let scale_factor = scale_factor.unwrap();
    let webviews_and_bounds = webviews.get_all_webviews_with_bounds();
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

        let window_size = LogicalSize::new(*width as f64, *height as f64);
        for (webview, bounds) in webviews_and_bounds.iter() {
            webview
                .set_bounds(bounds.to_webview_bounds(window_size, scale_factor.as_f64()))
                .unwrap();
        }
    }
}
