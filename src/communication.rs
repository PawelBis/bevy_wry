use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::{Deref, Event, Resource};
use serde::{Deserialize, Serialize};

#[derive(Event, Clone)]
pub enum OutEvent<T: Event + Clone + SerializeMessage> {
    Text(String),
    Event(T),
}

#[derive(Event)]
pub enum InEvent<T: Event + DeserializeMessage<Event = T>> {
    Text(String),
    Event(T),
}

#[derive(Deref, Resource)]
pub struct MessageBus<T: Send>(pub Arc<Mutex<Vec<T>>>);

impl<T: Send> MessageBus<T> {
    pub fn lock(&self) -> MutexGuard<Vec<T>> {
        self.0.lock().unwrap()
    }
}

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

    fn from_binary(_: Vec<u8>) -> Result<Self::Event, Self::Error>;
}

#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error),
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

pub trait SerializeMessage {
    type Error: std::fmt::Debug;

    fn to_binary(&self) -> Result<Vec<u8>, Self::Error>;
}

impl<T: Serialize> SerializeMessage for T {
    type Error = Error;

    fn to_binary(&self) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(self).map_err(Error::Bincode)
    }
}
