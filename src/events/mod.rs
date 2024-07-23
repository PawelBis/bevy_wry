use bevy::prelude::*;
use wry::{WebView, WebViewBuilder};

use crate::{
    communication::{
        types::{InWryEvent, MessageBus},
        ui::{Anchor, Bounds},
    },
    error::Error,
};

#[derive(Debug)]
pub enum Source {
    Html(String),
    Url(String),
}

#[derive(Event, Debug)]
pub enum WebViewEvent {
    /// Create a new WebView
    Create(CreateWebView),
    /// Update an anchor
    UpdateAnchor {
        webview_name: String,
        new_anchor: Anchor,
    },
    /// Update the bounds
    UpdateBounds {
        webview_name: String,
        new_bounds: Bounds,
    },
}

impl From<CreateWebView> for WebViewEvent {
    fn from(value: CreateWebView) -> Self {
        Self::Create(value)
    }
}

#[derive(Debug)]
pub struct CreateWebView {
    pub name: String,
    pub bounds: Bounds,
    pub source: Source,
    pub transparent: bool,
}

pub(crate) fn create_webview<I>(
    params: &CreateWebView,
    primary_window: &winit::window::Window,
    in_bus: MessageBus<I>,
) -> Result<WebView, Error>
where
    for<'de> I: InWryEvent<'de>,
{
    let size = primary_window.inner_size();
    let scale_factor = primary_window.scale_factor();
    let builder = WebViewBuilder::new_as_child(primary_window)
        .with_transparent(params.transparent)
        .with_bounds(params.bounds.to_webview_bounds(size, scale_factor));

    let builder = match &params.source {
        Source::Url(url) => builder.with_url(url.clone()),
        Source::Html(html) => builder.with_html(html.clone()),
    };

    Ok(builder
        .with_ipc_handler(move |request| {
            let event: I = serde_json::from_str(request.body()).unwrap();

            let mut in_bus = in_bus.lock();
            in_bus.push(event);
        })
        .build()?)
}

pub(crate) fn update_anchor(
    webview: &mut WebView,
    bounds: &mut Bounds,
    new_anchor: Anchor,
    primary_window: &winit::window::Window,
) -> Result<(), Error> {
    if let Bounds::Relative { bounds, anchor } = bounds {
        *anchor = new_anchor;
        webview.set_bounds(
            Bounds::Relative {
                anchor: *anchor,
                bounds: *bounds,
            }
            .to_webview_bounds(primary_window.inner_size(), primary_window.scale_factor()),
        )?;
        Ok(())
    } else {
        Err(Error::FailedToUpdateAnchor)
    }
}
