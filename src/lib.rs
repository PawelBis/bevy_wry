pub mod components;
mod error;
pub mod events;
pub mod systems;

use bevy::prelude::*;
use components::webview::WebViews;
use events::{InWryEvent, OutWryEvent};

use systems::events::{consume_ipfs_events, produce_out_scripts};
pub use wry;
pub use wry::dpi::{Position as WryPosition, Size as WrySize};

/// [Resource] storing url used by [wry::WebView].
// TODO: This can be modified to change the url at runtime.
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

pub struct BevyWryPlugin {
    setup_callback: fn(&mut App),
}

/// Register event type that will be triggered by incoming messages from [wry::WebView].
/// This function should be called in [BevyWryPlugin] setup callback.
///
/// Events registered this way can be received via observer pattern.
///
/// Example
/// ```rust
/// use bevy::prelude::*;
/// use bevy_wry::{register_incoming_event, BevyWryPlugin};
///
/// #[derive(Event, Clone, serde::Deserialize)]
/// enum IpfsInCommand {
///     Print(String),
/// }
///
/// fn run_app() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(BevyWryPlugin::new(|app| {
///             // Register our Event type
///             register_incoming_event::<IpfsInCommand>(app);
///         }))
///         // Add observer that will be handling events registered with
///         // [register_incoming_event]
///         .add_observer(handle_incoming_events)
///         .run();
/// }
///
/// fn handle_incoming_events(
///     // Triggered event
///     trigger: Trigger<IpfsInCommand>,
/// ) {
///    let event = trigger.event();
///    match event {
///         IpfsInCommand::Print(msg) => {
///             bevy::log::info!("Received message: {msg}");
///         }
///    }
/// }
/// ```
pub fn register_incoming_event<E>(app: &mut App)
where
    for<'de> E: InWryEvent<'de>,
{
    app.add_event::<E>()
        .add_systems(Update, consume_ipfs_events::<E>);
}

/// Register event type that will be sent to [wry::WebView].
/// This function should be called in [BevyWryPlugin] setup callback.
///
/// Events registered this will be received via observer pattern.
///
/// Example
/// ```rust
/// use bevy::prelude::*;
/// use bevy_wry::{register_out_event, BevyWryPlugin};
/// use bevy_wry::components::webview::WebViewComponent;
/// use bevy_wry::events::OutWryEvent;
///
/// // Simple event that will be calling `console.log` with provided message.
/// #[derive(Event, Clone, serde::Serialize)]
/// struct ConsoleLog(String);
/// impl OutWryEvent for ConsoleLog {
///     fn to_script(&self) -> String {
///         format!("console.log({})", self.0)
///     }
/// }
///
/// fn run_app() {
///     App::new()
///         .add_plugins(DefaultPlugins)
///         .add_plugins(BevyWryPlugin::new(|app| {
///             // Register our Event type
///             register_out_event::<ConsoleLog>(app);
///         }))
///         .add_systems(Update, send_commands)
///         .run();
/// }
///
/// fn send_commands(
///     mut commands: Commands,
///     query: Query<Entity, With<WebViewComponent>>,
/// ) {
///    let entity = query.single();
///    commands.trigger_targets(ConsoleLog("Hello from Bevy!".into()), entity);
/// }
/// ```
pub fn register_out_event<E: OutWryEvent>(app: &mut App) {
    app.add_event::<E>().add_observer(produce_out_scripts::<E>);
}

impl BevyWryPlugin {
    pub fn new(setup_callback: fn(&mut App)) -> Self {
        Self { setup_callback }
    }
}

impl Plugin for BevyWryPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .insert_non_send_resource(WebViews::default())
            .add_systems(
                Update,
                (
                    systems::webview::create_webviews,
                    systems::webview::keep_webviews_in_bounds,
                )
                    // FIXME: After wgpu update (22 -> 23), bevy commit: 4b05d2f4
                    // rendering doesn't work without this small delay. I narrowed
                    // it down to this wgpu commit: fb0cb1eb
                    .run_if(systems::boot_delay_elapsed),
            )
            .add_systems(PostUpdate, systems::events::clear_busses);

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        {
            // https://github.com/tauri-apps/tauri/issues/9304
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
            gtk::init().unwrap();

            // we need to ignore this error here otherwise it will be catched by winit and will be
            // make the example crash
            winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
                let error = error as *mut x11_dl::xlib::XErrorEvent;
                (unsafe { (*error).error_code }) == 170
            }));

            let app = app.add_systems(Update, gtk_iteration_do);
        }

        let setup = self.setup_callback;
        setup(app);
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
))]
fn gtk_iteration_do(_: &mut World) {
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}
