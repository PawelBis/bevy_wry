mod error;
mod websocket;

use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use error::Error;
use wry::{WebView, WebViewBuilder};

type Result<T> = std::result::Result<T, Error>;

/// Resource storing url data.
/// We use const generics here, so we can query urls separately
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

#[derive(Resource, Clone, Default)]
/// Wry window is allways spawned as a child of `PrimaryWindow`, otherwise
/// transparency in the webview will be broken.
pub struct BevyWryPlugin {
    /// WebView will be initialised with this url
    /// Additionally it will be stored via `insert_resource`
    pub url: UrlResource,
}

impl Plugin for BevyWryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .init_non_send_resource::<Option<WebView>>()
            .add_systems(Startup, setup_webview.map(utils::error))
            .add_systems(Startup, websocket::setup_websocket.map(utils::error));
    }
}

fn setup_webview(world: &mut World) -> Result<()> {
    let wry_config = world
        .remove_resource::<BevyWryPlugin>()
        .ok_or_else(|| Error::MissingResource("BevyWryPlugin".to_owned()))?;

    let primary_window_entity = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world);
    let primary_window = world
        .get_non_send_resource::<WinitWindows>()
        .ok_or_else(|| Error::MissingResource("WinitWindows".to_owned()))?
        .get_window(primary_window_entity)
        .ok_or(Error::FailedToGetMainWindow)?;

    let webview = WebViewBuilder::new_as_child(primary_window)
        .with_transparent(true)
        .with_url(&wry_config.url)?
        .with_bounds(wry::Rect {
            x: 0,
            y: 0,
            width: 1000,
            height: 1000,
        })
        .build()?;

    world.insert_resource(wry_config.url);
    world.insert_non_send_resource(webview);

    Ok(())
}
