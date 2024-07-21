use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to get main window")]
    FailedToGetMainWindow,
    #[error("wry error: {0}")]
    Wry(#[from] wry::Error),
}
