use wry::{
    dpi::{PhysicalPosition, PhysicalSize, Position},
    Rect,
};

#[derive(Debug, Clone)]
pub enum Anchor {
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

fn add_positions(position_a: &Position, position_b: &Position, scale_factor: f64) -> Position {
    let physical_a: PhysicalPosition<i32> = position_a.to_physical(scale_factor);
    let physical_b: PhysicalPosition<i32> = position_b.to_physical(scale_factor);

    PhysicalPosition {
        x: physical_a.x + physical_a.x,
        y: physical_b.y + physical_b.y,
    }
    .into()
}

/// Position of the WebView
#[derive(Debug, Clone)]
pub enum Bounds {
    /// WebView covers whole window
    FullScreen,
    /// Position and size doesn't change - always relative to the top left corner of the window
    Absolute(Rect),
    /// Position relative to the anchor point
    Relative { anchor: Anchor, bounds: Rect },
}

impl Bounds {
    pub fn to_webview_bounds(
        &self,
        window_width: f32,
        window_height: f32,
        scale_factor: f64,
    ) -> wry::Rect {
        match self {
            Bounds::FullScreen => wry::Rect {
                position: PhysicalPosition::new(0, 0).into(),
                size: PhysicalSize::new(window_width as u32, window_height as u32).into(),
            },
            Bounds::Absolute(rect) => *rect,
            Bounds::Relative { anchor, bounds } => {
                let center = PhysicalPosition {
                    x: ((window_width / 2.0) as u32),
                    y: ((window_height / 2.0) as u32),
                };
                let physical_size = bounds.size.to_physical(scale_factor);
                let relative_bounds = match anchor {
                    Anchor::Top => Rect {
                        position: PhysicalPosition { x: center.x, y: 0 }.into(),
                        size: bounds.size,
                    },
                    Anchor::Bottom => Rect {
                        position: PhysicalPosition {
                            x: center.x,
                            y: window_height as u32,
                        }
                        .into(),
                        size: bounds.size,
                    },
                    Anchor::Left => Rect {
                        position: PhysicalPosition { x: 0, y: center.y }.into(),
                        size: bounds.size,
                    },
                    Anchor::Right => Rect {
                        position: PhysicalPosition {
                            x: window_width as u32,
                            y: center.y,
                        }
                        .into(),
                        size: bounds.size,
                    },
                    Anchor::Center => Rect {
                        position: center.into(),
                        size: bounds.size,
                    },
                    Anchor::TopStretch => Rect {
                        position: PhysicalPosition { x: center.x, y: 0 }.into(),
                        size: PhysicalSize {
                            width: window_width,
                            height: physical_size.height,
                        }
                        .into(),
                    },
                    Anchor::BottomStretch => Rect {
                        position: PhysicalPosition {
                            x: center.x,
                            y: window_height as u32,
                        }
                        .into(),
                        size: PhysicalSize {
                            width: window_width,
                            height: physical_size.height,
                        }
                        .into(),
                    },
                    Anchor::LeftStretch => Rect {
                        position: PhysicalPosition { x: 0, y: center.y }.into(),
                        size: PhysicalSize {
                            width: physical_size.width,
                            height: window_height,
                        }
                        .into(),
                    },
                    Anchor::RightStretch => Rect {
                        position: PhysicalPosition {
                            x: window_width as u32,
                            y: center.y,
                        }
                        .into(),
                        size: PhysicalSize {
                            width: physical_size.width,
                            height: window_height,
                        }
                        .into(),
                    },
                    Anchor::TopLeft => Rect {
                        position: PhysicalPosition::new(0, 0).into(),
                        size: bounds.size,
                    },
                    Anchor::TopRight => Rect {
                        position: PhysicalPosition {
                            x: window_width as u32,
                            y: 0,
                        }
                        .into(),
                        size: bounds.size,
                    },
                    Anchor::BottomLeft => Rect {
                        position: PhysicalPosition {
                            x: 0,
                            y: window_height as u32,
                        }
                        .into(),
                        size: bounds.size,
                    },
                    Anchor::BottomRight => Rect {
                        position: PhysicalPosition {
                            x: window_width as u32,
                            y: window_height as u32,
                        }
                        .into(),
                        size: bounds.size,
                    },
                    Anchor::CenterVerticalStretch => Rect {
                        position: center.into(),
                        size: PhysicalSize {
                            width: physical_size.width,
                            height: window_height,
                        }
                        .into(),
                    },
                    Anchor::CenterHorizontalStretch => Rect {
                        position: center.into(),
                        size: PhysicalSize {
                            width: window_width,
                            height: physical_size.height,
                        }
                        .into(),
                    },
                };

                Rect {
                    position: add_positions(
                        &relative_bounds.position,
                        &bounds.position,
                        scale_factor,
                    ),
                    size: relative_bounds.size,
                }
            }
        }
    }
}
