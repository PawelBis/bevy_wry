use bevy::prelude::*;

use crate::communication::ui::Bounds;

#[derive(Event)]
pub struct CreateWebview {
    pub name: String,
    pub bounds: Bounds,
    pub url: Option<String>,
    pub transparent: bool,
}
