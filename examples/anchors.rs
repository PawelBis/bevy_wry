use bevy::app::AppExit;
use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use bevy_wry::components::webview::{Initialized, WebViewBundleBuilder};
use bevy_wry::components::Anchor;
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

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Srgba(PURPLE)))
        .add_event::<NextAnchor>()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::<InWrapper>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, handle_events)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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

    commands.spawn(
        WebViewBundleBuilder::new(WEBVIEW_NAME)
            .with_html(html)
            .with_transparent(false)
            .with_size(BTN_SIZE)
            .with_position(LogicalPosition::new(-50.0, -50.0))
            .with_anchor(Anchor::Center)
            .build(),
    );
}

fn handle_events(
    mut event_reader: EventReader<InWrapper>,
    mut exit_writer: EventWriter<AppExit>,
    mut webviews: Query<&mut Anchor, With<Initialized>>,
) {
    if webviews.is_empty() || event_reader.is_empty() {
        return;
    }

    event_reader.clear();
    let mut anchor = webviews.single_mut();
    let new_anchor = match *anchor {
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
        Anchor::CenterHorizontalStretch => Anchor::FullScreen,
        Anchor::FullScreen => {
            exit_writer.send(AppExit::Success);
            return;
        }
    };

    *anchor = new_anchor;
}
