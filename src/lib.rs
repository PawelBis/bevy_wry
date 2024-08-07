pub mod components;
mod error;
pub mod events;
pub mod systems;

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::utils;
use components::webview::WebViews;
use events::system::{consume_in_events, send_out_events};
use events::{EmptyInEvent, EmptyOutEvent, InWryEvent, MessageBus, OutWryEvent};

pub use wry;
pub use wry::dpi::{Position as WryPosition, Size as WrySize};

/// [Resource] storing url used by [WebView].
// TODO: This can be modified to change the url at runtime.
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

/// Creates a [WebView] window that can be used for both in game and editor UI rendering.
///
/// You can send events to webview via [EventWriter]<[OutEvent<Out>]> and read incoming
/// events with [EventReader]<[InEvent]>.
/// Out events are sent via webview.:evaluate_script.
pub struct BevyWryPlugin<I = EmptyInEvent, O = EmptyOutEvent>
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
        let _app = app
            .add_event::<I>()
            .add_event::<O>()
            .insert_non_send_resource(WebViews::default())
            .init_resource::<MessageBus<I>>()
            .init_resource::<MessageBus<O>>()
            .add_systems(Update, systems::create_webviews::<I>)
            .add_systems(Update, systems::keep_webviews_in_bounds)
            .add_systems(Update, consume_in_events::<I>)
            .add_systems(Update, send_out_events::<O>.map(utils::error));

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        {
            // https://github.com/tauri-apps/tauri/issues/9304
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            gtk::init().unwrap();

            // we need to ignore this error here otherwise it will be catched by winit and will be
            // make the example crash
            winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
                let error = error as *mut x11_dl::xlib::XErrorEvent;
                (unsafe { (*error).error_code }) == 170
            }));

            _app.add_systems(Update, gtk_iteration_do);
        }
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn gtk_iteration_do(_: &mut World) {
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}
