use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to get main window")]
    FailedToGetMainWindow,
    #[error("wry error: {0}")]
    Wry(#[from] wry::Error),
    #[error("webview with name '{0}' doesn't exist")]
    FailedToGetWebview(String),
    #[error("cannot update anchnor of a webview with not relative bounds")]
    FailedToUpdateAnchor,
}
