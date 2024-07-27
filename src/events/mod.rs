pub mod error;
pub mod system;

use wry::dpi::{LogicalPosition, LogicalSize, Position, Size};
use wry::{WebView, WebViewBuilder};

use crate::components::{Anchor, Bounds};
use crate::error::Error;

use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::{Deref, Event, Resource};
use serde::{Deserialize, Serialize};

pub trait OutWryEvent: Event + Serialize + Send {
    fn to_script(&self) -> String;
    fn target_webview(&self) -> Option<String> {
        None
    }
}

pub trait InWryEvent<'de>: Event + Deserialize<'de> + Send {}
impl<'de, T> InWryEvent<'de> for T where T: Event + Deserialize<'de> + Send {}

#[derive(Deserialize, Serialize, Event)]
pub struct EmptyOutEvent;

impl OutWryEvent for EmptyOutEvent {
    fn to_script(&self) -> String {
        "".to_string()
    }
}

#[derive(Deserialize, Serialize, Event)]
pub struct EmptyInEvent;

#[derive(Deref, Resource)]
pub struct MessageBus<T: Send>(pub Arc<Mutex<Vec<T>>>);

impl<T: Send> MessageBus<T> {
    pub fn lock(&self) -> MutexGuard<Vec<T>> {
        self.0.lock().unwrap()
    }
}

impl<T: Send> Default for MessageBus<T> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Vec::<T>::new())))
    }
}

impl<T: Send> Clone for MessageBus<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

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
    /// Close webview
    Close(String),
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

/// [CreateWebView] command builder.
///
/// Builds [Bounds]::FullScreen by default. Specify [Anchor] to build relative webview, or set
/// [Position] and/or [Size] to build a [Bounds]::Absolute webview.
///
/// [WebView] source can be specified with [CreateWebViewBuilder].with_url()/.with_html().
#[derive(Debug, Default)]
pub struct CreateWebViewBuilder {
    name: String,
    size: Option<Size>,
    position: Option<Position>,
    anchor: Option<Anchor>,
    source: Option<Source>,
    transparent: bool,
}

impl CreateWebViewBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set size. Default: 200x200
    pub fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Set position. Default: 0, 0
    pub fn with_position(mut self, position: impl Into<Position>) -> Self {
        self.position = Some(position.into());
        self
    }

    /// Set Anchor. Default: None
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = Some(anchor);
        self
    }

    /// Set webview source to Url
    pub fn with_url(mut self, url: String) -> Self {
        self.source = Some(Source::Url(url));
        self
    }

    /// Set webview source to Html
    pub fn with_html(mut self, html: String) -> Self {
        self.source = Some(Source::Html(html));
        self
    }

    /// Set transparency
    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.transparent = transparent;
        self
    }

    pub fn build(self) -> CreateWebView {
        let bounds = match (self.anchor, self.position, self.size) {
            (Some(anchor), position, size) => Bounds::Relative {
                anchor,
                bounds: wry::Rect {
                    position: position.unwrap_or(LogicalPosition::new(0.0, 0.0).into()),
                    size: size.unwrap_or(LogicalSize::new(200.0, 200.0).into()),
                },
            },
            (None, None, None) => Bounds::FullScreen,
            (None, position, size) => Bounds::Absolute(wry::Rect {
                position: position.unwrap_or(LogicalPosition::new(0.0, 0.0).into()),
                size: size.unwrap_or(LogicalSize::new(200.0, 200.0).into()),
            }),
        };

        CreateWebView {
            name: self.name,
            bounds,
            source: self.source.unwrap_or(Source::Html("".to_string())),
            transparent: self.transparent,
        }
    }
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
