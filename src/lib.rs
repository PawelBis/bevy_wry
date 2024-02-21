pub mod communication;
mod error;
pub mod websocket;
pub mod webview;

use std::marker::PhantomData;

use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use communication::{DeserializeMessage, InEvent, MessageBus, OutEvent, SerializeMessage};
use error::Error;
use tungstenite::Message;
use websocket::{consume_incoming_messages, send_outgoing_messages, setup_tcp_listener};
use webview::{keep_webview_fullscreen, ScaleFactor};
use wry::{WebView, WebViewBuilder};

type Result<T> = std::result::Result<T, Error>;

/// [Resource] storing url used by [WebView].
// TODO: This can be modified to change the url at runtime.
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

/// Convenience wrapper allowing usage of the same type for both in and out events.
///
/// See [BevyWryPlugin]
#[allow(unused)]
pub type SymmetricWryPlugin<E> = BevyWryPlugin<E, E>;

/// Convenience wrapper for when you don't care about communication with [WebView]
///
/// See [BevyWryPlugin].
pub type NakedWryPlugin = BevyWryPlugin<(), ()>;

/// Creates a [WebView] window that can be used for both in game and editor UI rendering.
///
/// Communication with webview windows is done via [tungstenite::WebSocket]. You can send
/// events to webview via [EventWriter]`<OutEvent<Out>>` and read incoming events with
/// [EventReader]`<InEvent<In>>`.
///
/// Please note that at the moment of writing this plugin relies heavily on [serde_json].
#[derive(Default, Resource)]
pub struct BevyWryPlugin<In, Out>
where
    In: Event + DeserializeMessage<Event = In>,
    // TODO: Use resource for Out events, so we can move instead of cloning
    Out: Event + SerializeMessage,
{
    /// Url loaded in the webview, stored in the 'UrlResource'
    pub url: UrlResource,
    /// [MessageBus] in which incoming messages are stored. All messages are transformed into [In]
    /// events.
    in_message_bus: MessageBus<InEvent<In>>,
    /// [MessageBus] in which outcoming messages are stored. This message bus is populated from
    /// events produced by [EventWriter]`<Out>`
    out_message_bus: MessageBus<Message>,
    _phantom_data: PhantomData<Out>,
}

impl<In, Out> Clone for BevyWryPlugin<In, Out>
where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage,
{
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            in_message_bus: self.in_message_bus.clone(),
            out_message_bus: self.out_message_bus.clone(),
            _phantom_data: PhantomData::<Out>,
        }
    }
}

impl<In, Out> BevyWryPlugin<In, Out>
where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage,
{
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: UrlResource(url.into()),
            in_message_bus: MessageBus::<InEvent<In>>::default(),
            out_message_bus: MessageBus::<Message>::default(),
            _phantom_data: PhantomData::<Out>,
        }
    }
}

impl<In, Out> Plugin for BevyWryPlugin<In, Out>
where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_event::<InEvent<In>>()
            .add_event::<OutEvent<Out>>()
            .init_non_send_resource::<Option<WebView>>()
            .add_systems(Startup, setup_webview::<In, Out>.map(utils::error))
            .add_systems(Update, keep_webview_fullscreen)
            .add_systems(Update, consume_incoming_messages::<InEvent<In>>)
            .add_systems(Update, send_outgoing_messages::<Out>);
    }
}

fn setup_webview<In, Out>(world: &mut World) -> Result<()>
where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage,
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
    world.insert_resource(ScaleFactor::from(scale_factor));
    world.insert_non_send_resource(webview);

    setup_tcp_listener::<In, Out>(in_bus, out_bus)?;

    Ok(())
}
