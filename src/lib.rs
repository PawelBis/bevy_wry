use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use wry::WebViewBuilder;

#[derive(Resource, Clone, Default)]
pub struct BevyWryPlugin {
    pub as_child: bool,
    pub url: Option<String>,
    pub html: Option<String>,
}

impl Plugin for BevyWryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_systems(Startup, setup_webview);
    }
}

fn setup_webview(world: &mut World) {
    let mut primary_window = world.query_filtered::<Entity, With<PrimaryWindow>>();
    let windows = world.get_non_send_resource::<WinitWindows>().unwrap();
    let raw_window = windows.get_window(primary_window.single(world)).unwrap();

    let wry_config = world.get_resource::<BevyWryPlugin>().unwrap();
    let mut webview_builder = if wry_config.as_child {
        WebViewBuilder::new_as_child(raw_window)
    } else {
        WebViewBuilder::new(raw_window)
    };

    if let Some(url) = &wry_config.url {
        webview_builder = webview_builder.with_url(&url).unwrap()
    };

    if let Some(html) = &wry_config.html {
        webview_builder = webview_builder.with_html(html).unwrap();
    }

    let webview = webview_builder.with_transparent(true).build().unwrap();
    world.insert_non_send_resource(webview);
}
