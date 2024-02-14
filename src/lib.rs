mod error;
pub mod websocket;

use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use error::Error;
use serde::Deserialize;
use websocket::{setup_websocket, MessageBus};
use wry::{WebView, WebViewBuilder};

type Result<T> = std::result::Result<T, Error>;

#[derive(Resource)]
pub struct ScaleFactor(f64);

impl ScaleFactor {
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

/// Resource storing url data.
/// We use const generics here, so we can query urls separately
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

/// Wry window is allways spawned as a child of `PrimaryWindow`, otherwise
/// transparency in the webview will be broken.
#[derive(Resource, Clone, Default)]
pub struct BevyWryPlugin<T: Send + 'static + Clone> {
    /// WebView will be initialised with this url
    /// Additionally it will be stored via `insert_resource`
    pub url: UrlResource,
    message_bus: MessageBus<T>,
}

impl<T: Send + 'static + Clone> BevyWryPlugin<T> {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: UrlResource(url.into()),
            message_bus: MessageBus::<T>::default(),
        }
    }
}

impl<'a, T: Send + 'static + Clone> Plugin for BevyWryPlugin<T>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .init_non_send_resource::<Option<WebView>>()
            .add_systems(Startup, setup_webview::<T>.map(utils::error));
    }
}

fn setup_webview<'a, T: Send + 'static + Clone>(world: &mut World) -> Result<()>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    let wry_config = world
        .remove_resource::<BevyWryPlugin<T>>()
        .ok_or_else(|| Error::MissingResource("BevyWryPlugin".to_owned()))?;

    let primary_window_entity = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world);
    let primary_window = world
        .get_non_send_resource::<WinitWindows>()
        .ok_or_else(|| Error::MissingResource("WinitWindows".to_owned()))?
        .get_window(primary_window_entity)
        .ok_or(Error::FailedToGetMainWindow)?;

    let scale_factor = primary_window.scale_factor();

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
    let message_bus = wry_config.message_bus;
    world.insert_resource(wry_config.url);
    world.insert_resource(message_bus.clone());
    world.insert_resource(ScaleFactor(scale_factor));
    world.insert_non_send_resource(webview);

    setup_websocket(message_bus)?;

    Ok(())
}
