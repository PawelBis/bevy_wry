# Leptos server side rendering

This example demonstrates how to use tokio, axum and leptos to levarage lapros's server side rendering capabilities. Please note how dependencies are specified in `Cargo.toml` - it is important to split `hydrate` (ui) and `ssr` (server) dependencies.

## Dependencies

- (cargo-leptos)[https://github.com/leptos-rs/cargo-leptos] - build server (game) and client code with one command
- wasm32 target - `wasm32-unknown-unknown`

## Running the example

```bash
cargo leptos serve
```
