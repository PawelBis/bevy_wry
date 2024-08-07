use bevy::prelude::*;

use crate::components::webview::WebViews;

use super::error::Error;
use super::{MessageBus, OutWryEvent};

pub fn consume_in_events<T: Event>(message_bus: ResMut<MessageBus<T>>, mut events: EventWriter<T>) {
    let messages = { message_bus.lock().split_off(0) };
    for msg in messages {
        events.send(msg);
    }
}

pub fn send_out_events<T: Event + OutWryEvent>(
    webviews: NonSend<WebViews>,
    mut events: EventReader<T>,
) -> Result<(), Error> {
    for event in events.read() {
        match event.target_webview() {
            Some(webview_name) => {
                if let Some(webview) = webviews.get_webview(&webview_name) {
                    webview
                        .evaluate_script(&event.to_script())
                        .map_err(|_| Error::EvaluateScript)?;
                }
            }
            None => {
                for webview in webviews.get_all() {
                    webview
                        .evaluate_script(&event.to_script())
                        .map_err(|_| Error::EvaluateScript)?;
                }
            }
        }
    }

    Ok(())
}
