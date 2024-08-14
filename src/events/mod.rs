pub mod error;

use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait OutWryEvent: Event + Serialize + Send {
    fn to_script(&self) -> String;
}

pub trait InWryEvent<'de>: Event + Deserialize<'de> + Send {}
impl<'de, T> InWryEvent<'de> for T where T: Event + Deserialize<'de> + Send {}

#[derive(Deserialize, Serialize, Event)]
pub struct EmptyOutEvent;

#[derive(Deserialize, Serialize, Event)]
pub struct EmptyInEvent;

/// MessageBus gathers all webview events.
#[derive(Deref)]
pub struct MessageBus {
    messages: Arc<RwLock<Vec<String>>>,
}

impl MessageBus {
    pub fn write(&self) -> RwLockWriteGuard<Vec<String>> {
        self.messages.write().unwrap()
    }

    pub fn read(&self) -> RwLockReadGuard<Vec<String>> {
        self.messages.read().unwrap()
    }

    pub fn clear(&self) {
        self.messages.write().unwrap().clear();
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Clone for MessageBus {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
        }
    }
}

#[derive(Component, Deref, Default)]
pub struct InMessageBus(MessageBus);

#[derive(Component, Deref, Default)]
pub struct OutMessageBus(MessageBus);
