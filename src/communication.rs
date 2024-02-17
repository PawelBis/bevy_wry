use std::sync::{Arc, Mutex};

use bevy::prelude::{Deref, Event, Resource};
use serde::{Deserialize, Serialize};

#[repr(u8)]
pub enum Direction {
    In = 0,
    Out = 1,
}
pub const IN: u8 = Direction::In as u8;
pub const OUT: u8 = Direction::In as u8;

impl From<Direction> for u8 {
    fn from(v: Direction) -> Self {
        v as u8
    }
}

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
