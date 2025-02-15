use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use std::ops::Deref;
use wry::WebViewBuilder;

use crate::components::bounds::{to_webview_bounds, Position, Size};
use crate::components::webview::{Initialized, Source, Transparency, WebViewComponent, WebViews};
use crate::components::Anchor;
use crate::events::{InMessageBus, InWryEvent, OutMessageBus, OutWryEvent};

#[allow(clippy::type_complexity)]
pub fn create_webviews(
    mut commands: Commands,
    mut webviews: NonSendMut<WebViews>,
    webview_entities: Query<
        (
            Entity,
            &WebViewComponent,
            &Anchor,
            &Position,
            &Size,
            &Source,
            &Transparency,
        ),
        Without<Initialized>,
    >,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
) {
    let primary_window = primary_window_entity.single();
    let primary_window = winit_windows.get_window(primary_window).unwrap();
    let window_size = primary_window.inner_size();
    let scale_factor = primary_window.scale_factor();

    for (entity, webview_component, anchor, position, size, source, transparency) in
        webview_entities.iter()
    {
        let bounds = to_webview_bounds(*anchor, position.0, size.0, window_size, scale_factor);
        let builder = WebViewBuilder::new()
            .with_transparent(transparency.0)
            .with_bounds(bounds);

        let builder = match source {
            Source::Url(url) => builder.with_url(url.clone()),
            Source::Html(html) => builder.with_html(html.clone()),
        };

        let in_bus = InMessageBus::default();
        let ipc_bus = in_bus.clone();
        let webview = builder
            .with_ipc_handler(move |request| {
                ipc_bus.write().push(request.body().clone());
            })
            .build_as_child(primary_window.deref())
            .unwrap();

        let WebViewComponent { webview_name } = webview_component;
        webviews.insert(webview_name.clone(), webview);

        commands
            .entity(entity)
            .insert(Initialized)
            .insert(in_bus)
            .insert(OutMessageBus::default());
    }
}

pub fn keep_webviews_in_bounds(
    webviews: NonSendMut<WebViews>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
    webview_entities: Query<(&WebViewComponent, &Position, &Size, &Anchor), With<Initialized>>,
) {
    let primary_window = primary_window_entity.single();
    let winit_window = winit_windows.get_window(primary_window).unwrap();
    let scale_factor = winit_window.scale_factor();
    let window_size = winit_window.inner_size();
    for (webview_component, position, size, anchor) in webview_entities.iter() {
        let WebViewComponent { webview_name } = webview_component;
        let webview = webviews
            .get_webview(webview_name)
            .expect("WebView with 'Initialized' component should be present in WebViews resource");
        let bounds = to_webview_bounds(*anchor, position.0, size.0, window_size, scale_factor);
        webview.set_bounds(bounds).unwrap();
    }
}

/// Reads `MessageBus`, converts the json to `E` event and triggers
/// the event. Those events can be read with observer systems.
pub fn trigger_webview_event<T>(mut commands: Commands, webviews: Query<(Entity, &InMessageBus)>)
where
    for<'de> T: InWryEvent<'de>,
{
    for (entity, msg_bus) in webviews.iter() {
        for msg in msg_bus.read().iter() {
            let event: T = serde_json::from_str(msg).unwrap();
            commands.trigger_targets(event, entity);
        }
    }
}

/// Observes out events and propagates those to OutMessageBus
pub fn out_events<E: OutWryEvent>(trigger: Trigger<E>, out_bus: Query<&OutMessageBus>) {
    let ob = out_bus.get(trigger.entity()).unwrap();
    let event: &E = trigger.event();
    ob.write().push(event.to_script());
}

pub fn clear_busses(
    webviews: NonSend<WebViews>,
    busses: Query<(&WebViewComponent, &InMessageBus, &OutMessageBus)>,
) {
    for (webview_component, in_bus, out_bus) in busses.iter() {
        for msg in out_bus.read().iter() {
            webviews
                .get_webview(&webview_component.webview_name)
                .unwrap()
                .evaluate_script(msg)
                .unwrap();
        }

        in_bus.clear();
        out_bus.clear();
    }
}
