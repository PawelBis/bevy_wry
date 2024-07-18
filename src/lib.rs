pub mod communication;
mod error;
pub mod webview;

use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use communication::types::{InWryEvent, MessageBus, OutWryEvent};
use communication::{consume_in_events, send_out_events};
use error::Error;
use webview::{keep_webview_fullscreen, ScaleFactor};
use wry::dpi::{PhysicalPosition, PhysicalSize, Position, Size};
use wry::{Rect, WebView, WebViewBuilder};

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
/// You can send events to webview via [EventWriter]<[OutEvent<Out>]> and read incoming
/// events with [EventReader]<[InEvent]>.
/// Out events are sent via webview.:evaluate_script.
#[derive(Default, Resource)]
pub struct BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    /// Url loaded in the webview, stored in the 'UrlResource'
    pub url: UrlResource,
    /// [MessageBus] in which incoming messages are stored.
    in_message_bus: MessageBus<I>,
    /// [MessageBus] in which outcoming messages are stored. This message bus is populated from
    /// events produced by [EventWriter]`<Out>`
    out_message_bus: MessageBus<O>,
}

impl<I, O> Clone for BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    fn clone(&self) -> Self {
        Self {
            url: self.url.clone(),
            in_message_bus: self.in_message_bus.clone(),
            out_message_bus: self.out_message_bus.clone(),
        }
    }
}

impl<I, O> BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: UrlResource(url.into()),
            in_message_bus: MessageBus::<I>::default(),
            out_message_bus: MessageBus::<O>::default(),
        }
    }
}

impl<I, O> Plugin for BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone())
            .add_event::<I>()
            .add_event::<O>()
            .init_non_send_resource::<Option<WebView>>()
            .add_systems(Startup, setup_webview::<I, O>.map(utils::error))
            .add_systems(Update, keep_webview_fullscreen)
            .add_systems(Update, consume_in_events::<I>)
            .add_systems(Update, send_out_events::<O>.map(utils::error));
    }
}

fn setup_webview<I, O>(world: &mut World) -> Result<()>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    let wry_config = world
        .remove_resource::<BevyWryPlugin<I, O>>()
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

    let in_bus = wry_config.in_message_bus;
    let in_bus_handler = in_bus.clone();

    let webview = WebViewBuilder::new_as_child(primary_window)
        .with_transparent(true)
        .with_url(&wry_config.url.0)
        .with_bounds(Rect {
            position: Position::new(PhysicalPosition::new(0, 0)),
            size: Size::new(PhysicalSize::new(1000, 1000)),
        })
        .with_ipc_handler(move |request| {
            let event: I = serde_json::from_str(request.body()).unwrap();

            let mut in_bus = in_bus_handler.lock();
            in_bus.push(event);
        })
        .build()?;

    let in_bus_resource = in_bus.clone();
    world.insert_resource(in_bus_resource);

    let out_bus = wry_config.out_message_bus;
    let out_bus_resource = out_bus.clone();
    world.insert_resource(out_bus_resource);

    world.insert_resource(wry_config.url);
    world.insert_resource(ScaleFactor::from(scale_factor));
    world.insert_non_send_resource(webview);

    Ok(())
}
