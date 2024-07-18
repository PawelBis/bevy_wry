#[derive(Debug)]
pub enum Error {
    Deserialize,
    BadMessageType,
    CloseRequested,
    EvaluateScript,
}
