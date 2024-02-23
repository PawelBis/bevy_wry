use std::sync::{Arc, Mutex, MutexGuard};

use super::Error;
use bevy::prelude::{Deref, Event, Resource};
use tungstenite::Message;

#[derive(Event)]
pub enum OutEvent<T: Event + SerializeMessage> {
    Text(String),
    Event(T),
}

impl<T: Event + SerializeMessage> OutEvent<T> {
    pub fn to_message(&self) -> Result<Message, Error> {
        let msg = match self {
            OutEvent::Text(text) => Message::Text(text.clone()),
            OutEvent::Event(event) => {
                Message::Binary(event.to_binary().map_err(|_| Error::Deserialize)?)
            }
        };

        Ok(msg)
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

pub trait SerializeMessage {
    type Error: std::fmt::Debug;

    fn to_binary(&self) -> Result<Vec<u8>, Self::Error>;
}
