pub mod components;
mod error;
pub mod events;
pub mod systems;

use bevy::prelude::*;
use components::webview::WebViews;
use events::{InWryEvent, OutWryEvent};

use systems::{out_events, trigger_webview_event};
pub use wry;
pub use wry::dpi::{Position as WryPosition, Size as WrySize};

/// [Resource] storing url used by [WebView].
// TODO: This can be modified to change the url at runtime.
#[derive(Resource, Deref, Clone, Default)]
pub struct UrlResource(pub String);

pub struct BevyWryPlugin {
    setup_callback: fn(&mut App),
}

impl BevyWryPlugin {
    pub fn new(setup_callback: fn(&mut App)) -> Self {
        Self { setup_callback }
    }

    pub fn reqister_in_webview_event<E>(app: &mut App)
    where
        for<'de> E: InWryEvent<'de>,
    {
        app.add_event::<E>()
            .add_systems(Update, trigger_webview_event::<E>);
    }

    pub fn reqister_out_webview_event<E: OutWryEvent>(app: &mut App) {
        app.add_event::<E>().add_observer(out_events::<E>);
    }
}

impl Plugin for BevyWryPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .insert_non_send_resource(WebViews::default())
            .add_systems(
                Update,
                (systems::create_webviews, systems::keep_webviews_in_bounds)
                    // FIXME: After wgpu update (22 -> 23), bevy commit: 4b05d2f4
                    // rendering doesn't work without this small delay. I narrowed
                    // it down to this wgpu commit: fb0cb1eb
                    .run_if(systems::boot_delay_elapsed),
            )
            .add_systems(PostUpdate, systems::clear_busses);

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
