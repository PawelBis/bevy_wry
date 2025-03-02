use axum::Router;
use bevy::color::palettes::css::PURPLE;
use bevy::log;
use bevy::prelude::*;
use bevy_wry::components::webview::WebViewBundleBuilder;
use bevy_wry::BevyWryPlugin;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use leptos_ssr::app::*;
use tokio::runtime::Runtime;

#[allow(unused)]
struct Tokio {
    tokio_runtime: Runtime,
    axum_handle: tokio::task::JoinHandle<()>,
    url: String,
}

/// This function initializes tokio runtime and spawns axum server task
/// Drop the handle to stop the server
fn spawn_axum_server() -> Tokio {
    let tokio_runtime = Runtime::new().unwrap();
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let address = format!("http://{addr}");
    let handle = tokio_runtime.spawn(async move {
        let addr = conf.leptos_options.site_addr;
        let leptos_options = conf.leptos_options;
        // Generate the list of routes in your Leptos App
        let routes = generate_route_list(App);

        let app = Router::new()
            .leptos_routes(&leptos_options, routes, {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            })
            .fallback(leptos_axum::file_and_error_handler(shell))
            .with_state(leptos_options);

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        log::info!("listening on {address}");
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });

    let address = format!("http://{addr}");
    return Tokio {
        tokio_runtime,
        axum_handle: handle,
        url: address,
    };
}

fn setup(mut commands: Commands, axum_server_address: Res<AxumServerAddress>) {
    commands.spawn(
        WebViewBundleBuilder::new("AXUM")
            .with_transparent(true)
            .with_url(axum_server_address.0.clone())
            .build(),
    );
}

#[derive(Resource, Debug)]
struct AxumServerAddress(String);

#[cfg(feature = "ssr")]
fn main() {
    let tokio = spawn_axum_server();
    App::new()
        .insert_resource(ClearColor(Color::Srgba(PURPLE)))
        .insert_resource(AxumServerAddress(tokio.url))
        .add_plugins(DefaultPlugins)
        .add_plugins(BevyWryPlugin::new(|_| {}))
        .add_systems(Startup, setup)
        //.add_observer(in_commands)
        .run();
}
