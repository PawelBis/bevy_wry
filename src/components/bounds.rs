use crate::{WryPosition, WrySize};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use winit::dpi::Size as WinitSize;
use wry::dpi::{LogicalPosition, LogicalSize};
use wry::Rect;

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Component)]
pub enum Anchor {
    /// Stretch to fill whole window
    FullScreen,
    /// Maintain position relative to the top edge of the window
    Top,
    /// Maintain position relative to the bottom edge of the window
    Bottom,
    /// Maintain position relative to the left edge of the window
    Left,
    /// Maintain position relative to the right edge of the window
    Right,
    /// Maintain position relative to the center of the window
    Center,
    /// Stretch on the top edge of the window
    TopStretch,
    /// Stretch on the bottom edge of the window
    BottomStretch,
    /// Stretch on the left edge of the window
    LeftStretch,
    /// Stretch on the right edge of the window
    RightStretch,
    /// Maintain position relative to the top left corner of the window
    TopLeft,
    /// Maintain position relative to the top right corner of the window
    TopRight,
    /// Maintain position relative to the bottom left corner of the window
    BottomLeft,
    /// Maintain position relative to the bottom right corner of the window
    BottomRight,
    /// Stretch from the bottom edge to the top edge, maintaining position relative to the center
    CenterVerticalStretch,
    /// Stretch from the left edge to the right edge, maintaining position relative to the center
    CenterHorizontalStretch,
}

#[derive(Component, Debug)]
pub struct Position(pub WryPosition);

#[derive(Component, Debug)]
pub struct Size(pub WrySize);

fn add_positions(
    position_a: &WryPosition,
    position_b: &WryPosition,
    scale_factor: f64,
) -> WryPosition {
    let physical_a: LogicalPosition<f64> = position_a.to_logical(scale_factor);
    let physical_b: LogicalPosition<f64> = position_b.to_logical(scale_factor);

    LogicalPosition {
        x: physical_a.x + physical_b.x,
        y: physical_a.y + physical_b.y,
    }
    .into()
}

pub fn to_webview_bounds(
    anchor: Anchor,
    position: WryPosition,
    size: WrySize,
    window_size: impl Into<WinitSize>,
    scale_factor: f64,
) -> Rect {
    let (window_width, window_height) = match window_size.into() {
        WinitSize::Physical(ps) => {
            let ls = ps.to_logical(scale_factor);
            (ls.width, ls.height)
        }
        WinitSize::Logical(ls) => (ls.width, ls.height),
    };

    let center = LogicalPosition {
        x: ((window_width / 2.0) as u32),
        y: ((window_height / 2.0) as u32),
    };
    let current_size = size.to_logical(scale_factor);
    let relative_bounds = match anchor {
        Anchor::FullScreen => {
            return Rect {
                position: LogicalPosition::new(0, 0).into(),
                size: LogicalSize::new(window_width as u32, window_height as u32).into(),
            }
        }
        Anchor::Top => Rect {
            position: LogicalPosition { x: center.x, y: 0 }.into(),
            size,
        },
        Anchor::Bottom => Rect {
            position: LogicalPosition {
                x: center.x,
                y: window_height as u32,
            }
            .into(),
            size,
        },
        Anchor::Left => Rect {
            position: LogicalPosition { x: 0, y: center.y }.into(),
            size,
        },
        Anchor::Right => Rect {
            position: LogicalPosition {
                x: window_width as u32,
                y: center.y,
            }
            .into(),
            size,
        },
        Anchor::Center => Rect {
            position: center.into(),
            size,
        },
        Anchor::TopStretch => Rect {
            position: LogicalPosition { x: 0, y: 0 }.into(),
            size: LogicalSize {
                width: window_width,
                height: current_size.height,
            }
            .into(),
        },
        Anchor::BottomStretch => Rect {
            position: LogicalPosition {
                x: 0,
                y: (window_height - current_size.height) as u32,
            }
            .into(),
            size: LogicalSize {
                width: window_width,
                height: current_size.height,
            }
            .into(),
        },
        Anchor::LeftStretch => Rect {
            position: LogicalPosition { x: 0, y: 0 }.into(),
            size: LogicalSize {
                width: current_size.width,
                height: window_height,
            }
            .into(),
        },
        Anchor::RightStretch => Rect {
            position: LogicalPosition {
                x: (window_width - current_size.width) as u32,
                y: 0,
            }
            .into(),
            size: LogicalSize {
                width: current_size.width,
                height: window_height,
            }
            .into(),
        },
        Anchor::TopLeft => Rect {
            position: LogicalPosition::new(0, 0).into(),
            size,
        },
        Anchor::TopRight => Rect {
            position: LogicalPosition {
                x: window_width as u32,
                y: 0,
            }
            .into(),
            size,
        },
        Anchor::BottomLeft => Rect {
            position: LogicalPosition {
                x: 0,
                y: window_height as u32,
            }
            .into(),
            size,
        },
        Anchor::BottomRight => Rect {
            position: LogicalPosition {
                x: window_width as u32,
                y: window_height as u32,
            }
            .into(),
            size,
        },
        Anchor::CenterVerticalStretch => Rect {
            position: LogicalPosition { x: center.x, y: 0 }.into(),
            size: LogicalSize {
                width: current_size.width,
                height: window_height,
            }
            .into(),
        },
        Anchor::CenterHorizontalStretch => Rect {
            position: LogicalPosition { x: 0, y: center.y }.into(),
            size: LogicalSize {
                width: window_width,
                height: current_size.height,
            }
            .into(),
        },
    };

    match anchor {
        Anchor::TopStretch
        | Anchor::BottomStretch
        | Anchor::LeftStretch
        | Anchor::RightStretch
        | Anchor::CenterVerticalStretch
        | Anchor::CenterHorizontalStretch => relative_bounds,
        _ => Rect {
            position: add_positions(&relative_bounds.position, &position, scale_factor),
            size: relative_bounds.size,
        },
    }
}
