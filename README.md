# About

'bevy_wry' is a [bevy](https://github.com/bevyengine/bevy/) plugin that provides integration with [wry](https://github.com/tauri-apps/wry) - a cross platform webview rendering library written in rust.

BevyWry relies on bevy@0.14 [observer pattern](https://bevyengine.org/examples/ecs-entity-component-system/observers/).
- Events can be sent to specific WebView via 'commands.trigger_targets'
- Events can be received via observer system, observing for 'Trigger<OutEventType>'

This plugin is in EARLY and EXPERIMENTAL stage.

Please keep in mind that you will have to add this patch to use `bevy_wry`:
```
[patch.crates-io]
# At the moment http disallows empty authority and invalidates uris like: "file:///path/to/file"
http = { git = "https://github.com/PawelBis/http", branch = "feature/empty-authority" }
```

# Examples

- [fullscreen](https://github.com/PawelBis/bevy_wry/blob/main/examples/fullscreen.rs) - how to create and use fullscreen webview with transparency. Linux and Windows doesn't support transparency at the moment
- [anchors](https://github.com/PawelBis/bevy_wry/blob/main/examples/anchors.rs)

Run an example with:
`cargo run --example=example_name`
