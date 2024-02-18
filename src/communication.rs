use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::{Deref, Event, Resource};
use serde::{Deserialize, Serialize};
use tungstenite::Message;

#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error),
    Deserialize,
    BadMessageType,
    CloseRequested,
}

#[derive(Event, Clone)]
pub enum OutEvent<T: Event + Clone + SerializeMessage> {
    Text(String),
    Event(T),
}

impl<T> TryInto<Message> for OutEvent<T>
where
    T: Event + SerializeMessage + Clone,
{
    type Error = Error;

    fn try_into(self) -> Result<Message, Self::Error> {
        Ok(match self {
            OutEvent::Text(text) => Message::Text(text),
            OutEvent::Event(event) => {
                Message::Binary(event.to_binary().map_err(|_| Error::Deserialize)?)
            }
        })
    }
}

#[derive(Event)]
pub enum InEvent<T: Event + DeserializeMessage<Event = T>> {
    Text(String),
    Event(T),
}

impl<T> TryFrom<Message> for InEvent<T>
where
    T: Event + DeserializeMessage<Event = T>,
{
    type Error = Error;

    fn try_from(value: Message) -> Result<Self, Self::Error> {
        match value {
            Message::Text(text) => Ok(Self::Text(text)),
            Message::Binary(buffer) => Ok(Self::Event(
                T::from_binary(buffer).map_err(|_| Error::Deserialize)?,
            )),
            Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => Err(Error::BadMessageType),
            Message::Close(_) => Err(Error::CloseRequested),
        }
    }
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
