use bevy::app::AppExit;
use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use bevy_wry::components::webview::WebViewBundleBuilder;
use bevy_wry::events::OutWryEvent;
use bevy_wry::BevyWryPlugin;
use std::env;

const WEBVIEW_NAME: &str = "MAIN_WEBVIEW";

/// Command arriving from WebView
#[derive(Event, Clone, serde::Deserialize)]
enum InCommand {
    Rotate { angle: f32 },
    Exit,
}

/// Command send to webview
#[derive(Event, Clone, serde::Serialize)]
enum OutCommand {
    ShowButton,
}

impl OutWryEvent for OutCommand {
    fn to_script(&self) -> String {
        match self {
            OutCommand::ShowButton => "showButton()".to_string(),
        }
    }
}

fn init_bevy_wry(app: &mut App) {
    BevyWryPlugin::reqister_in_webview_event::<InCommand>(app);
    BevyWryPlugin::reqister_out_webview_event::<OutCommand>(app);
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::Srgba(PURPLE)))
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::new(init_bevy_wry))
        .add_systems(Startup, setup)
        .add_observer(in_commands)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d::default());
    commands.spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..default()
        },
        Transform::from_translation(Vec3::new(-50.0, -50.0, 0.0)),
    ));

    // bevy_wry needs absolute path to files for now
    let manifest_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_path = format!("file://{manifest_path}/examples/web/index.html");
    commands.spawn(
        WebViewBundleBuilder::new(WEBVIEW_NAME)
            .with_transparent(true)
            .with_url(ui_path)
            .build(),
    );
}

fn in_commands(
    trigger: Trigger<InCommand>,
    mut commands: Commands,
    mut exit_writer: EventWriter<AppExit>,
    mut sprite: Query<(&mut Transform, &Sprite)>,
) {
    let event = trigger.event();
    let webview_entity = trigger.entity();

    match event {
        InCommand::Rotate { angle } => {
            let (mut transform, _) = sprite.single_mut();
            transform.rotate_z(f32::to_radians(*angle));

            let (_, z) = transform.rotation.to_axis_angle();
            if z == f32::to_radians(180.0) {
                commands.trigger_targets(OutCommand::ShowButton, webview_entity);
            }
        }
        InCommand::Exit => {
            exit_writer.send(AppExit::Success);
        }
    }
}
