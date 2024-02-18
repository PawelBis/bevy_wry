use std::sync::{Arc, Mutex};

use bevy::prelude::{Deref, Event, Resource};
use serde::{Deserialize, Serialize};

#[derive(Deref, Event, Serialize)]
#[serde(transparent)]
pub struct OutEvent<T: Event + Clone + Serialize>(pub T);

impl<T: Event + Clone + Serialize> Clone for OutEvent<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[derive(Deref, Event, Deserialize)]
#[serde(transparent)]
pub struct InEvent<T: Event>(pub T);

#[derive(Deref, Resource)]
pub struct MessageBus<T: Send>(pub Arc<Mutex<Vec<T>>>);

impl<T: Send> Default for MessageBus<T> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Vec::<T>::new())))
    }
}

impl<T: Send> Clone for MessageBus<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait DeserializeMessage {
    type Event;
    type Error: std::fmt::Debug;

    fn from_string(_: String) -> Result<Self::Event, Self::Error>;
    fn from_binary(_: Vec<u8>) -> Result<Self::Event, Self::Error>;
}

#[derive(Debug)]
pub enum Error {
    SerdeJson(serde_json::Error),
    Bincode(bincode::Error),
}

impl<T> DeserializeMessage for T
where
    for<'de> T: Deserialize<'de>,
{
    type Error = Error;

    type Event = Self;

    fn from_string(string: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&string).map_err(Error::SerdeJson)
    }

    fn from_binary(buffer: Vec<u8>) -> Result<Self::Event, Self::Error> {
        bincode::deserialize(&buffer).map_err(Error::Bincode)
    }
}

pub trait SerializeMessage {
    type Error: std::fmt::Debug;

    fn to_string(&self) -> Result<String, Self::Error>;
    fn to_binary(&self) -> Result<Vec<u8>, Self::Error>;
}

impl<T: Serialize> SerializeMessage for T {
    type Error = Error;

    fn to_string(&self) -> Result<String, Self::Error> {
        serde_json::to_string(self).map_err(Error::SerdeJson)
    }

    fn to_binary(&self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(self).map_err(Error::Bincode)
    }
}
