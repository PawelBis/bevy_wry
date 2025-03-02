use bevy::prelude::*;

use crate::components::webview::{WebViewComponent, WebViews};
use crate::events::{InMessageBus, InWryEvent, OutMessageBus, OutWryEvent};

/// Consume all messages from [InMessageBus] and trigger corresponding events.
///
/// Messages stored in [InMessageBus] are received via webview IPC mechanism.
/// Events triggered this way can be received via observer pattern.
pub(crate) fn consume_ipfs_events<E>(
    mut commands: Commands,
    webviews: Query<(Entity, &InMessageBus)>,
) where
    for<'de> E: InWryEvent<'de>,
{
    for (entity, msg_bus) in webviews.iter() {
        for msg in msg_bus.read().iter() {
            let event: E = serde_json::from_str(msg).unwrap();
            commands.trigger_targets(event, entity);
        }
    }
}

/// Consume incoming [OutWryEvent] and push it's script representation to [OutMessageBus].
pub(crate) fn produce_out_scripts<E: OutWryEvent>(
    trigger: Trigger<E>,
    out_bus: Query<&OutMessageBus>,
) {
    let ob = out_bus.get(trigger.entity()).unwrap();
    let event: &E = trigger.event();
    ob.write().push(event.to_script());
}

pub(crate) fn clear_busses(
    webviews: NonSend<WebViews>,
    busses: Query<(&WebViewComponent, &InMessageBus, &OutMessageBus)>,
) {
    for (webview_component, in_bus, out_bus) in busses.iter() {
        for msg in out_bus.read().iter() {
            webviews
                .get_webview(&webview_component.webview_name)
                .unwrap()
                .evaluate_script(msg)
                .unwrap();
        }

        in_bus.clear();
        out_bus.clear();
    }
}
