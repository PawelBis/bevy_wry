use thiserror::Error;

use crate::websocket;

#[derive(Error, Debug)]
pub enum Error {
    #[error("missing `{0}` resource")]
    MissingResource(String),
    #[error("failed to get main window")]
    FailedToGetMainWindow,
    #[error("wry error: {0}")]
    Wry(#[from] wry::Error),
    #[error("websocket error: {0}")]
    Websocket(#[from] websocket::Error),
}
