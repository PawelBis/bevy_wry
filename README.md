# About

'bevy_wry' is a [bevy](https://github.com/bevyengine/bevy/) plugin that provides integration with [wry](https://github.com/tauri-apps/wry) - cross platform webview rendering library written in rust.

'bevy_wry' enables [bevy::Event](https://docs.rs/bevy/latest/bevy/ecs/event/trait.Event.html) based communication with WebView through [websocket](https://github.com/snapview/tungstenite-rs/).

It is still in very early stages, however I think it is good enough for some experimentation.

Each client is simply reading/writing to websocket in a thread through [`MessageBus`](https://github.com/PawelBis/bevy_wry/blob/main/src/communication.rs#L62). The 'websocket.read()' call is non blocking - current version is relying on [`TcpStream::set_non_clocking(true)`](https://doc.rust-lang.org/std/net/struct.TcpStream.html#method.set_nonblocking), however this will be improved in the future, as current implementation is quite expensive.

You can read events incoming from websocket with [`EventReader<InEvent<T>>`](https://docs.rs/bevy/latest/bevy/ecs/event/struct.EventReader.html) and write events with [`EventWriter<OutEvent<T>>`](https://docs.rs/bevy/latest/bevy/ecs/event/struct.EventWriter.html).

# Example

Check cargo run --example [simple](https://github.com/PawelBis/bevy_wry/blob/main/examples/simple.rs) for a quick reference.
