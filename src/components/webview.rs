use crate::{WryPosition, WrySize};
use bevy::prelude::*;
use bevy::utils::hashbrown::hash_map::Values;
use bevy::utils::hashbrown::HashMap;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::WebView;

use crate::error::Error;

use super::bounds::{Position, Size};
use super::Anchor;

#[derive(Component, Debug)]
pub struct WebViewComponent {
    pub webview_name: String,
}

impl WebViewComponent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            webview_name: name.into(),
        }
    }
}

#[derive(Debug, Component)]
pub enum Source {
    Html(String),
    Url(String),
}

#[derive(Debug, Component)]
pub struct Transparency(pub bool);

#[derive(Debug, Component)]
pub struct Fullscreen(pub bool);

#[derive(Bundle)]
pub struct WebViewBundle {
    pub webview: WebViewComponent,
    pub anchor: Anchor,
    pub position: Position,
    pub size: Size,
    pub source: Source,
    pub transparency: Transparency,
}

#[derive(Debug, Default)]
pub struct WebViewBundleBuilder {
    /// Main webview component
    webview: Option<WebViewComponent>,
    /// Webview size. Default: fullscreen
    size: Option<Size>,
    /// Webview position. Default: 0, 0
    position: Option<Position>,
    /// Webview anchor. Default [Anchor::FullScreen]
    anchor: Option<Anchor>,
    /// Webview source. Either Html or Url. Default: Source::Html("")
    source: Option<Source>,
    /// Transparent webview. Default: false
    transparent: Option<bool>,
}

impl WebViewBundleBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            webview: Some(WebViewComponent::new(name)),
            ..Default::default()
        }
    }

    /// Set size. Default: 200x200
    pub fn with_size(mut self, size: impl Into<WrySize>) -> Self {
        self.size = Some(Size(size.into()));
        self
    }

    /// Set position. Default: 0, 0
    pub fn with_position(mut self, position: impl Into<WryPosition>) -> Self {
        self.position = Some(Position(position.into()));
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
        self.transparent = Some(transparent);
        self
    }

    pub fn build(self) -> WebViewBundle {
        let position = self
            .position
            .unwrap_or(Position(LogicalPosition::new(0.0, 0.0).into()));
        let size = self.size.unwrap_or(Size(LogicalSize::new(0.0, 0.0).into()));
        let source = self.source.unwrap_or(Source::Html("".to_string()));
        let transparency = Transparency(self.transparent.unwrap_or(false));
        let anchor = self.anchor.unwrap_or(Anchor::FullScreen);

        WebViewBundle {
            webview: self.webview.expect("WebView builder sets name"),
            position,
            size,
            source,
            transparency,
            anchor,
        }
    }
}

#[derive(Component)]
pub struct Initialized;

#[derive(Default)]
pub struct WebViews {
    /// TODO: Use HashMap<Entity, WebView> instead
    webviews: HashMap<String, WebView>,
}

impl WebViews {
    pub fn insert(&mut self, name: String, webview: WebView) {
        self.webviews.insert(name.clone(), webview);
    }

    pub fn get_webview(&self, name: &String) -> Option<&WebView> {
        self.webviews.get(name)
    }

    pub fn get_all(&self) -> Values<'_, String, WebView> {
        self.webviews.values()
    }

    pub fn remove_webview(&mut self, name: &String) -> Result<(), Error> {
        self.webviews
            .remove(name)
            .ok_or_else(|| Error::FailedToGetWebview(name.clone()))?;

        Ok(())
    }
}
