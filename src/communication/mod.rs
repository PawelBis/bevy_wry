pub mod types;

pub mod error;
use bevy::prelude::{Event, EventReader, EventWriter, NonSend, ResMut};
pub use error::Error;
use wry::WebView;

use self::types::{MessageBus, OutWryEvent};

pub fn consume_in_events<T: Event>(message_bus: ResMut<MessageBus<T>>, mut events: EventWriter<T>) {
    let messages = { message_bus.lock().split_off(0) };
    for msg in messages {
        events.send(msg);
    }
}

pub fn send_out_events<T: Event + OutWryEvent>(
    webview: NonSend<WebView>,
    mut events: EventReader<T>,
) -> Result<(), Error> {
    for event in events.read() {
        webview
            .evaluate_script(&event.to_script())
            .map_err(|_| Error::EvaluateScript)?;
    }

    Ok(())
}
