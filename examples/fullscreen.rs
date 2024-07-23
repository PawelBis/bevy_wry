use bevy::{app::AppExit, prelude::*};
use bevy_wry::{
    communication::{types::OutWryEvent, ui::Bounds},
    events::{CreateWebView, WebViewEvent},
    BevyWryPlugin,
};
use std::env;

const WEBVIEW_NAME: &str = "MAIN_WEBVIEW";

#[derive(Event, Clone, serde::Serialize, serde::Deserialize)]
enum Command {
    Rotate { angle: f32 },
    ShowButton,
    Exit,
}

#[derive(Event, Clone, serde::Serialize, serde::Deserialize)]
struct InWrapper(pub Command);

#[derive(Event, Clone, serde::Serialize, serde::Deserialize)]
struct OutWrapper(pub Command);

impl OutWryEvent for OutWrapper {
    fn to_script(&self) -> String {
        match self.0 {
            // ShowButton is our only OutCommand
            // Please note that 'showButton' is a method implemented in
            // our UI code: examples/web/ui.html
            Command::ShowButton => "showButton()".to_string(),
            _ => unreachable!(),
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::PURPLE))
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::<InWrapper, OutWrapper>::default())
        .add_systems(Startup, setup)
        .add_systems(Update, handle_events)
        .run();
}

fn setup(mut commands: Commands, mut writer: EventWriter<WebViewEvent>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-50.0, -50.0, 0.0)),
        ..default()
    });

    // bevy_wry needs absolute path to files for now
    let manifest_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_path = format!("file://{manifest_path}/examples/web/ui.html");
    writer.send(
        CreateWebView {
            name: WEBVIEW_NAME.to_string(),
            source: bevy_wry::events::Source::Url(ui_path),
            transparent: true,
            bounds: Bounds::FullScreen,
        }
        .into(),
    );
}

fn handle_events(
    mut event_reader: EventReader<InWrapper>,
    mut event_writer: EventWriter<OutWrapper>,
    mut exit_writer: EventWriter<AppExit>,
    mut sprite: Query<(&mut Transform, &Sprite)>,
) {
    for event in event_reader.read() {
        match event.0 {
            Command::Rotate { angle } => {
                let (mut transform, _) = sprite.single_mut();
                transform.rotate_z(f32::to_radians(angle));

                let (_, z) = transform.rotation.to_axis_angle();
                if z == f32::to_radians(180.0) {
                    event_writer.send(OutWrapper(Command::ShowButton));
                }
            }
            Command::Exit => {
                exit_writer.send(AppExit);
            }
            _ => (),
        }
    }
}
