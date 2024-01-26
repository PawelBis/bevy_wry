use bevy::{prelude::*, window::PrimaryWindow, winit::WinitWindows};
use wry::{raw_window_handle::HasRawWindowHandle, Rect, WebView, WebViewBuilder};

/// Resource storing url data.
/// We use const generics here, so we can query urls separately
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource<const N: u8>(pub String);

#[derive(Resource, Clone, Copy, Deref, Debug)]
pub struct WebViewBounds(pub Rect);

impl From<Rect> for WebViewBounds {
    fn from(value: Rect) -> Self {
        Self(value)
    }
}

impl Default for WebViewBounds {
    fn default() -> Self {
        Rect {
            x: 0,
            y: 0,
            width: 200,
            height: 200,
        }
        .into()
    }
}

#[derive(Resource, Clone, Default)]
pub struct BevyWryPlugin {
    /// WebView will be created as a child window if `as_child == true`
    pub as_child: bool,
    /// WebView will be initialised with this bounds.
    /// Default: Rect { x: 0, y: 0, width: 200, height: 200 }
    pub bounds: Option<WebViewBounds>,
    /// WebView will be initialised with this url
    /// Additionally it will be stored via `insert_resource`
    pub url: UrlResource<0>,
}

impl Plugin for BevyWryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .init_non_send_resource::<Option<WebView>>()
            .add_systems(Startup, setup_webview)
            .add_systems(Last, update_webview_bounds.after(setup_webview));
    }
}

fn init_webview_builder<'a>(
    wry_config: &BevyWryPlugin,
    parent: &'a impl HasRawWindowHandle,
) -> WebViewBuilder<'a> {
    if wry_config.as_child {
        WebViewBuilder::new_as_child(parent)
            .with_bounds(*wry_config.bounds.clone().unwrap_or_default())
    } else {
        WebViewBuilder::new(parent)
    }
}

fn setup_webview(world: &mut World) {
    let wry_config = world
        .remove_resource::<BevyWryPlugin>()
        .expect("BevyWryPlugin resource is missing");

    let primary_window_entity = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world);
    let primary_window = world
        .get_non_send_resource::<WinitWindows>()
        .and_then(|windows| windows.get_window(primary_window_entity))
        .expect("Couldn't get primary window");

    let webview = init_webview_builder(&wry_config, primary_window)
        .with_transparent(true)
        .with_url(&wry_config.url)
        .and_then(|wb| wb.build())
        .expect("Failed to build webview");

    world.insert_resource(wry_config.url);
    world.insert_resource(wry_config.bounds.unwrap_or_default());
    world.insert_non_send_resource(webview);
}

/// This system handles changes in webview bounds. Those changes can be schedules via `WebViewBounds` resource
fn update_webview_bounds(bounds: Res<WebViewBounds>, webview: NonSend<WebView>) {
    let bounds: WebViewBounds = *bounds;
    webview.set_bounds(*bounds);
}
