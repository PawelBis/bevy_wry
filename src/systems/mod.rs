use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::WinitWindows;
use std::ops::Deref;
use wry::WebViewBuilder;

use crate::components::bounds::{to_webview_bounds, Position, Size};
use crate::components::webview::{Initialized, Source, Transparency, WebViewComponent, WebViews};
use crate::components::Anchor;
use crate::events::{InWryEvent, MessageBus};

#[allow(clippy::type_complexity)]
pub fn create_webviews<I>(
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
    in_messages: Res<MessageBus<I>>,
    primary_window_entity: Query<Entity, With<PrimaryWindow>>,
    winit_windows: NonSend<WinitWindows>,
) where
    for<'de> I: InWryEvent<'de>,
{
    let primary_window = primary_window_entity.single();
    let primary_window = winit_windows.get_window(primary_window).unwrap();
    let window_size = primary_window.inner_size();
    let scale_factor = primary_window.scale_factor();

    for (entity, webview_component, anchor, position, size, source, transparency) in
        webview_entities.iter()
    {
        let bounds = to_webview_bounds(*anchor, position.0, size.0, window_size, scale_factor);
        let builder = WebViewBuilder::new_as_child(primary_window.deref())
            .with_transparent(transparency.0)
            .with_bounds(bounds);

        let builder = match source {
            Source::Url(url) => builder.with_url(url.clone()),
            Source::Html(html) => builder.with_html(html.clone()),
        };

        let in_bus = in_messages.clone();
        let webview = builder
            .with_ipc_handler(move |request| {
                let event: I = serde_json::from_str(request.body()).unwrap();
                in_bus.lock().push(event);
            })
            .build()
            .unwrap();

        let WebViewComponent { webview_name } = webview_component;
        webviews.insert(webview_name.clone(), webview);

        commands.entity(entity).insert(Initialized);
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
