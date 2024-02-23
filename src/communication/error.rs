#[derive(Debug)]
pub enum Error {
    #[cfg(feature = "bincode")]
    Bincode(bincode::Error),
    Deserialize,
    BadMessageType,
    CloseRequested,
}
