pub mod communication;
mod error;
pub mod events;
pub mod webview;

use std::marker::PhantomData;

use bevy::ecs::system::SystemState;
use bevy::{prelude::*, utils, window::PrimaryWindow, winit::WinitWindows};
use communication::types::{InWryEvent, MessageBus, OutWryEvent, EmptyOutEvent, EmptyInEvent};
use communication::{consume_in_events, send_out_events};
use error::Error;
use events::{create_webview, update_anchor, WebViewEvent};
use webview::{keep_webviews_in_bounds, ScaleFactor, WebViews};

type Result<T> = std::result::Result<T, Error>;

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
            .add_event::<WebViewEvent>()
            .insert_non_send_resource(WebViews::default())
            .init_resource::<MessageBus<I>>()
            .init_resource::<MessageBus<O>>()
            .insert_resource(ScaleFactor::from(1.0))
            .add_systems(Update, handle_webview_events::<I>.map(utils::error))
            .add_systems(Update, keep_webviews_in_bounds)
            .add_systems(Update, consume_in_events::<I>)
            .add_systems(Update, send_out_events::<O>.map(utils::error));
    }
}

fn handle_webview_events<I>(world: &mut World) -> Result<()>
where
    for<'de> I: InWryEvent<'de>,
{
    let mut system_state = SystemState::<(
        EventReader<WebViewEvent>,
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

    for event in create_webview_events.read() {
        let event: &WebViewEvent = event;
        match event {
            WebViewEvent::Create(params) => {
                let webview = create_webview(params, primary_window, in_bus.clone())?;

                webviews.insert(params.name.clone(), webview, params.bounds.clone());
            }
            WebViewEvent::UpdateAnchor {
                webview_name,
                new_anchor,
            } => {
                let (webview, bounds) = webviews
                    .get_webview_with_bounds_mut(webview_name)
                    .ok_or_else(|| Error::FailedToGetWebview(webview_name.clone()))?;
                update_anchor(webview, bounds, *new_anchor, primary_window)?;
            }
            WebViewEvent::UpdateBounds {
                webview_name,
                new_bounds,
            } => {
                let (webview, bounds) = webviews
                    .get_webview_with_bounds_mut(webview_name)
                    .ok_or_else(|| Error::FailedToGetWebview(webview_name.clone()))?;
                *bounds = new_bounds.clone();
                webview.set_bounds(bounds.to_webview_bounds(
                    primary_window.inner_size(),
                    primary_window.scale_factor(),
                ))?;
            }
            WebViewEvent::Close(name) => {
                webviews.remove_webview(name)?;
            }
        };
    }
    create_webview_events.clear();

    world.insert_resource(ScaleFactor::from(scale_factor));
    Ok(())
}
