use std::sync::{Arc, Mutex, MutexGuard};

use bevy::prelude::{Deref, Event, Resource};
use serde::{Deserialize, Serialize};

pub trait OutWryEvent: Event + Serialize + Send {
    fn to_script(&self) -> String;
    fn target_webview(&self) -> Option<String> {
        None
    }
}

pub trait InWryEvent<'de>: Event + Deserialize<'de> + Send {}
impl<'de, T> InWryEvent<'de> for T where T: Event + Deserialize<'de> + Send {}

#[derive(Deserialize, Serialize, Event)]
pub struct EmptyOutEvent;

impl OutWryEvent for EmptyOutEvent {
    fn to_script(&self) -> String { "".to_string() }
}

#[derive(Deserialize, Serialize, Event)]
pub struct EmptyInEvent;

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
