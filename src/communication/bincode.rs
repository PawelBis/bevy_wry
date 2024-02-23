use serde::{Deserialize, Serialize};

use super::types::{DeserializeMessage, SerializeMessage};
use super::Error;

impl<T: Serialize> SerializeMessage for T {
    type Error = Error;

    fn to_binary(&self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(self).map_err(Error::Bincode)
    }
}

impl<T> DeserializeMessage for T
where
    for<'de> T: Deserialize<'de>,
{
    type Error = Error;
    type Event = Self;

    fn from_binary(buffer: Vec<u8>) -> Result<Self::Event, Self::Error> {
        bincode::deserialize(&buffer).map_err(Error::Bincode)
    }
}
