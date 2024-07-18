# About

'bevy_wry' is a [bevy](https://github.com/bevyengine/bevy/) plugin that provides integration with [wry](https://github.com/tauri-apps/wry) - a cross platform webview rendering library written in rust.

BevyWry allows for [bevy::Event](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html) based communication with WebView:
- Out events are required to implement [OutWryEvent]. This allows for [wry::WebView::evaluate_script](https://docs.rs/wry/latest/wry/struct.WebView.html#method.evaluate_script) communication with WebView
- Incoming events are received via IPC channel registered with [wry::WebViewBuilder::with_ipc_handler](https://docs.rs/wry/latest/wry/struct.WebViewBuilder.html#method.with_ipc_handler)

This plugin is still in very early stages, but it should be good enough for somne experimental work.

# Example

Check the [simple](https://github.com/PawelBis/bevy_wry/blob/main/examples/simple.rs) example for a quick reference.
`cargo run --example simple --features="simple-example"`
