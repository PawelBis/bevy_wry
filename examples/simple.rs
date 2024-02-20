use bevy::{app::AppExit, prelude::*};
use bevy_wry::{
    communication::{InEvent, OutEvent},
    BevyWryPlugin,
};
use std::env;

#[derive(Event, Clone, serde::Serialize, serde::Deserialize)]
enum Command {
    Rotate { angle: f32 },
    ShowButton,
    Exit,
}

fn main() {
    // bevy_wry needs absolute path to files for now
    let manifest_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_path = format!("file://{manifest_path}/examples/web/ui.html");

    App::new()
        .insert_resource(ClearColor(Color::PURPLE))
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::<Command, Command>::new(ui_path))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_events)
        .run();
}

fn setup(mut commands: Commands) {
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
}

fn handle_events(
    mut event_reader: EventReader<InEvent<Command>>,
    mut event_writer: EventWriter<OutEvent<Command>>,
    mut exit_writer: EventWriter<AppExit>,
    mut sprite: Query<(&mut Transform, &Sprite)>,
) {
    for event in event_reader.read() {
        if let InEvent::Text(string) = event {
            let command: Command = match serde_json::from_str(&string) {
                Ok(c) => c,
                Err(_) => {
                    info!(string);
                    return;
                }
            };

            let (mut transform, _) = sprite.single_mut();
            match command {
                Command::Rotate { angle } => {
                    transform.rotate_z(f32::to_radians(angle));

                    let (_, z) = transform.rotation.to_axis_angle();
                    if z == f32::to_radians(180.0) {
                        let show_btn_command = serde_json::to_string(&Command::ShowButton).unwrap();
                        event_writer.send(OutEvent::Text(show_btn_command));
                    }
                }
                Command::Exit => {
                    exit_writer.send(AppExit);
                }
                _ => (),
            }
        }
    }
}
