pub mod communication;
mod error;
pub mod websocket;

use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use communication::{InEvent, MessageBus, OutEvent};
use error::Error;
use serde::{Deserialize, Serialize};
use websocket::{consume_incoming_messages, setup_tcp_listener};
use wry::{WebView, WebViewBuilder};

type Result<T> = std::result::Result<T, Error>;

#[derive(Resource)]
pub struct ScaleFactor(f64);

impl ScaleFactor {
    pub fn as_f64(&self) -> f64 {
        self.0
    }
}

/// Resource storing url used by webview.
/// This can be modified to change the url at runtime.
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

#[allow(unused)]
pub type SymmetricWryPlugin<E> = BevyWryPlugin<InEvent<E>, OutEvent<E>>;

/// Wry window is allways spawned as a child of `PrimaryWindow`, otherwise
/// transparency in the webview will be broken.
#[derive(Resource, Default)]
pub struct BevyWryPlugin<In, Out>
where
    In: Event,
    for<'de> In: Deserialize<'de>,
    // TODO: Use resource for Out events, so we can move instead of cloning
    Out: Event + Serialize + Clone,
{
    /// Url loaded in the webview, stored in the 'UrlResource'
    pub url: UrlResource,
    /// Message bus used for incoming messages
    in_message_bus: MessageBus<In>,
    /// Message bus used for outgoing messages
    out_message_bus: MessageBus<Out>,
}

impl<In, Out> Clone for BevyWryPlugin<In, Out>
where
    In: Event,
    for<'de> In: Deserialize<'de>,
    Out: Event + Serialize + Clone,
{
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            in_message_bus: self.in_message_bus.clone(),
            out_message_bus: self.out_message_bus.clone(),
        }
    }
}

impl<In, Out> BevyWryPlugin<In, Out>
where
    In: Event,
    for<'de> In: Deserialize<'de>,
    Out: Event + Serialize + Clone,
{
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: UrlResource(url.into()),
            in_message_bus: MessageBus::<In>::default(),
            out_message_bus: MessageBus::<Out>::default(),
        }
    }
}

impl<In, Out> Plugin for BevyWryPlugin<In, Out>
where
    In: Event,
    for<'de> In: Deserialize<'de>,
    Out: Event + Serialize + Clone,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_event::<In>()
            .init_non_send_resource::<Option<WebView>>()
            .add_systems(Startup, setup_webview::<In, Out>.map(utils::error))
            .add_systems(Update, consume_incoming_messages::<In>);
    }
}

fn setup_webview<In, Out>(world: &mut World) -> Result<()>
where
    In: Event,
    for<'de> In: Deserialize<'de>,
    Out: Event + Serialize + Clone,
{
    let wry_config = world
        .remove_resource::<BevyWryPlugin<In, Out>>()
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

    let in_bus = wry_config.in_message_bus;
    let in_bus_resource = in_bus.clone();
    world.insert_resource(in_bus_resource);

    let out_bus = wry_config.out_message_bus;
    let out_bus_resource = out_bus.clone();
    world.insert_resource(out_bus_resource);

    world.insert_resource(wry_config.url);
    world.insert_resource(ScaleFactor(scale_factor));
    world.insert_non_send_resource(webview);

    setup_tcp_listener(in_bus, out_bus)?;

    Ok(())
}
