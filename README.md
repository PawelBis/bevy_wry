# About

'bevy_wry' is a [bevy](https://github.com/bevyengine/bevy/) plugin that provides integration with [wry](https://github.com/tauri-apps/wry) - a cross platform webview rendering library written in rust.

BevyWry allows for [bevy::Event](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html) based communication with WebView:
- Out events are required to implement [OutWryEvent]. This allows for [wry::WebView::evaluate_script](https://docs.rs/wry/latest/wry/struct.WebView.html#method.evaluate_script) communication with WebView
- Incoming events are received via IPC channel registered with [wry::WebViewBuilder::with_ipc_handler](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_ipc_handler)
This plugin is in EARLY and EXPERIMENTAL stage.

# Examples

- [fullscreen](https://github.com/PawelBis/bevy_wry/blob/main/examples/fullscreen.rs) - how to create and use fullscreen webview with transparency. Linux and Windows doesn't support transparency at the moment
- [anchors](https://github.com/PawelBis/bevy_wry/blob/main/examples/anchors.rs)

Run an example with:
`cargo run --example=example_name`
