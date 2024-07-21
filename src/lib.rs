pub mod communication;
mod error;
pub mod events;
pub mod webview;

use std::marker::PhantomData;

use bevy::ecs::system::SystemState;
use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use communication::types::{InWryEvent, MessageBus, OutWryEvent};
use communication::{consume_in_events, send_out_events};
use error::Error;
use events::CreateWebview;
use webview::{keep_webviews_in_bounds, ScaleFactor, WebViews};
use wry::WebViewBuilder;

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
pub struct BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    _i: PhantomData<I>,
    _o: PhantomData<O>,
}

impl<I, O> Default for BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    fn default() -> Self {
        Self {
            _i: PhantomData,
            _o: PhantomData,
        }
    }
}

impl<I, O> Plugin for BevyWryPlugin<I, O>
where
    for<'de> I: InWryEvent<'de>,
    O: OutWryEvent,
{
    fn build(&self, app: &mut App) {
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        {
            gtk::init().unwrap();

            // we need to ignore this error here otherwise it will be catched by winit and will be
            // make the example crash
            winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
                let error = error as *mut x11_dl::xlib::XErrorEvent;
                (unsafe { (*error).error_code }) == 170
            }));
        }

        app.add_event::<I>()
            .add_event::<O>()
            .add_event::<CreateWebview>()
            .insert_non_send_resource(WebViews::default())
            .init_resource::<MessageBus<I>>()
            .init_resource::<MessageBus<O>>()
            .add_systems(Update, create_webview::<I>.map(utils::error))
            .add_systems(Update, keep_webviews_in_bounds)
            .add_systems(Update, consume_in_events::<I>)
            .add_systems(Update, send_out_events::<O>.map(utils::error));
    }
}

fn create_webview<I>(world: &mut World) -> Result<()>
where
    for<'de> I: InWryEvent<'de>,
{
    let mut system_state = SystemState::<(
        EventReader<CreateWebview>,
        Query<Entity, With<PrimaryWindow>>,
        NonSend<WinitWindows>,
        Res<MessageBus<I>>,
        NonSendMut<WebViews>,
    )>::new(world);

    let (mut create_webview_events, primary_window_entity, winit_windows, in_bus, mut webviews) =
        system_state.get_mut(world);

    let primary_window_entity = primary_window_entity.single();

    let primary_window: &winit::window::Window = winit_windows
        .get_window(primary_window_entity)
        .ok_or(Error::FailedToGetMainWindow)?;

    let scale_factor = primary_window.scale_factor();
    let size = primary_window.inner_size();

    for event in create_webview_events.read() {
        let mut builder = WebViewBuilder::new_as_child(primary_window)
            .with_transparent(event.transparent)
            .with_bounds(event.bounds.to_webview_bounds(
                size.width as f32,
                size.height as f32,
                scale_factor,
            ));

        if let Some(url) = event.url.clone() {
            builder = builder.with_url(url);
        }

        let in_bus = in_bus.clone();
        let webview = builder
            .with_ipc_handler(move |request| {
                let event: I = serde_json::from_str(request.body()).unwrap();

                let mut in_bus = in_bus.lock();
                in_bus.push(event);
            })
            .build()?;

        webviews.insert(event.name.clone(), webview, event.bounds.clone());
    }

    world.insert_resource(ScaleFactor::from(scale_factor));
    Ok(())
}
