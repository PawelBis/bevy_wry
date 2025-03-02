use bevy::prelude::*;

pub mod events;
pub mod webview;

pub fn boot_delay_elapsed(time: Res<Time>) -> bool {
    time.elapsed_secs() > 1.0
}
