use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_wry::components::{Anchor, Bounds};
use bevy_wry::events::{CreateWebView, WebViewEvent};
use bevy_wry::BevyWryPlugin;
use wry::dpi::{LogicalPosition, LogicalSize};

const WEBVIEW_NAME: &str = "MAIN_WEBVIEW";
const BTN_SIZE: LogicalSize<f64> = LogicalSize {
    width: 100.0,
    height: 100.0,
};

/// This example uses only one event
#[derive(Event, Clone, serde::Serialize, serde::Deserialize)]
struct NextAnchor;

#[derive(Event, Clone, serde::Serialize, serde::Deserialize)]
struct InWrapper(pub NextAnchor);

#[derive(Resource)]
struct CurrentAnchor(Anchor);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::PURPLE))
        .insert_resource(CurrentAnchor(Anchor::Center))
        .add_event::<NextAnchor>()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::<InWrapper>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, handle_events)
        .run();
}

fn setup(mut writer: EventWriter<WebViewEvent>) {
    let html = r#"
      <html>
        <style>
          html, body {
            margin: 0;
            padding: 0;
          }
          button {
            display: block;
            width: 100%;
            height: 100%;
          }
        </style>
        <body style="background-color:red;">
          <button onclick="nextAnchor()">Catch Me</button>
        </body>
        <script>
          function nextAnchor() {
            <!-- null will be deserialized to NextAnchor -->
            console.log("ASDF");
            window.ipc.postMessage("null");
          }
        </script>
      </html>
    "#
    .to_string();

    writer.send(
        CreateWebView {
            name: WEBVIEW_NAME.to_string(),
            source: bevy_wry::events::Source::Html(html),
            transparent: true,
            bounds: Bounds::Relative {
                anchor: Anchor::Center,
                bounds: wry::Rect {
                    position: LogicalPosition::new(-50.0, -50.0).into(),
                    size: BTN_SIZE.into(),
                },
            },
        }
        .into(),
    );
}

fn handle_events(
    mut event_reader: EventReader<InWrapper>,
    mut exit_writer: EventWriter<AppExit>,
    mut webview_event_writer: EventWriter<WebViewEvent>,
    mut current_anchor: ResMut<CurrentAnchor>,
) {
    for _ in event_reader.read() {
        let new_anchor = match current_anchor.0 {
            Anchor::Top => Anchor::TopRight,
            Anchor::Bottom => Anchor::BottomLeft,
            Anchor::Left => Anchor::TopLeft,
            Anchor::Right => Anchor::BottomRight,
            Anchor::Center => Anchor::Top,
            Anchor::TopStretch => Anchor::RightStretch,
            Anchor::BottomStretch => Anchor::LeftStretch,
            Anchor::LeftStretch => Anchor::CenterVerticalStretch,
            Anchor::RightStretch => Anchor::BottomStretch,
            Anchor::TopLeft => Anchor::TopStretch,
            Anchor::TopRight => Anchor::Right,
            Anchor::BottomLeft => Anchor::Left,
            Anchor::BottomRight => Anchor::Bottom,
            Anchor::CenterVerticalStretch => Anchor::CenterHorizontalStretch,
            Anchor::CenterHorizontalStretch => Anchor::Center,
        };

        if new_anchor == Anchor::Center {
            exit_writer.send(AppExit);
        }

        *current_anchor = CurrentAnchor(new_anchor);
        webview_event_writer.send(WebViewEvent::UpdateAnchor {
            webview_name: WEBVIEW_NAME.to_string(),
            new_anchor,
        });
    }
}
