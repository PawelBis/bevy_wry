use bevy::prelude::*;
use bevy_wry::components::webview::WebViewBundleBuilder;
use bevy_wry::BevyWryPlugin;
use std::env;
use std::process::{Child, Command, Stdio};

fn setup(mut commands: Commands) {
    commands.spawn(
        WebViewBundleBuilder::new("WV_NAME")
            .with_transparent(true)
            .with_url("http://localhost:8080".to_string())
            .build(),
    );
}

struct CommandGuard(Child);
impl Drop for CommandGuard {
    fn drop(&mut self) {
        match self.0.kill() {
            Ok(_) => println!("Trunk closed"),
            Err(e) => println!("Failed to kill child process: {e}"),
        }
    }
}

fn main() {
    let manifest_path = env::var("CARGO_MANIFEST_DIR").unwrap();
    let ui_path = format!("{manifest_path}/examples/leptos/ui");

    // Use trunk to build `ui` which is a leptos project
    Command::new("trunk")
        .arg("build")
        .current_dir(&ui_path)
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();

    // Use trunk to serve `ui` which is a leptos project
    let o = Command::new("trunk")
        .arg("serve")
        .current_dir(ui_path)
        .spawn()
        .unwrap();

    let _g = CommandGuard(o);
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::new(|_| {}))
        .add_systems(Startup, setup)
        .run();
}
