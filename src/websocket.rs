use crate::communication::{DeserializeMessage, InEvent, MessageBus, OutEvent, SerializeMessage};
use bevy::prelude::*;
use std::net::{AddrParseError, TcpListener, TcpStream};
use std::thread;
use thiserror;
use tungstenite::util::NonBlockingError;
use tungstenite::{Message, WebSocket};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse address: {0}")]
    FailedToBincTcpListener(#[from] AddrParseError),
}

pub fn setup_tcp_listener<In, Out>(
    in_bus: MessageBus<InEvent<In>>,
    out_bus: MessageBus<OutEvent<Out>>,
) -> Result<(), Error>
where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage + Clone,
{
    let server = TcpListener::bind("localhost:8876").unwrap();
    server.set_nonblocking(true).unwrap();

    let ib = in_bus.clone();
    let ob = out_bus.clone();
    thread::spawn(move || handle_connections(server, ib, ob));

    Ok(())
}

fn handle_connections<In, Out>(
    server: TcpListener,
    in_bus: MessageBus<InEvent<In>>,
    out_bus: MessageBus<OutEvent<Out>>,
) where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage + Clone,
{
    loop {
        for stream in server.incoming() {
            let ib = in_bus.clone();
            let ob = out_bus.clone();
            thread::spawn(move || {
                if let Ok(s) = stream {
                    handle_client(s, ib, ob)
                }
            });
        }
    }
}

/// Accept incoming connections and spawn thread reading/writing to websocket.
/// This fn assumes that 'TcpListener' is running in 'non_blocking' mode.
fn handle_client<In, Out>(
    stream: TcpStream,
    in_bus: MessageBus<InEvent<In>>,
    out_bus: MessageBus<OutEvent<Out>>,
) where
    In: Event + DeserializeMessage<Event = In>,
    Out: Event + SerializeMessage + Clone,
{
    let mut socket = loop {
        match tungstenite::accept(&stream) {
            Ok(socket) => break socket,
            Err(e) => match e {
                tungstenite::HandshakeError::Interrupted(_) => (),
                tungstenite::HandshakeError::Failure(_) => panic!("Websocket Handshake failed"),
            },
        };
    };

    loop {
        if try_read_messages(&mut socket, &in_bus) {
            break;
        }
        try_write_messages(&mut socket, &out_bus);
    }
}

///
fn try_read_messages<In>(
    socket: &mut WebSocket<&TcpStream>,
    in_bus: &MessageBus<InEvent<In>>,
) -> bool
where
    In: Event + DeserializeMessage<Event = In>,
{
    match socket.read() {
        Ok(msg) => {
            let decoded_event = match msg {
                Message::Text(string) => InEvent::Text(string),
                Message::Binary(buffer) => InEvent::Event(In::from_binary(buffer).unwrap()),
                Message::Close(_) => return true,
                Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => return false,
            };

            let mut msg_bus = in_bus.lock();
            msg_bus.push(decoded_event);
        }
        Err(e) => {
            if e.into_non_blocking().is_some() {
                return true;
            }
        }
    }

    false
}

fn try_write_messages<Out>(socket: &mut WebSocket<&TcpStream>, out_bus: &MessageBus<OutEvent<Out>>)
where
    Out: Event + SerializeMessage + Clone,
{
    let out_events = match out_bus.try_lock() {
        Ok(mut lock) => Some(lock.split_off(0)),
        Err(e) => match e {
            std::sync::TryLockError::Poisoned(_) => panic!("Poisoned mutex"),
            std::sync::TryLockError::WouldBlock => None,
        },
    };

    if let Some(events) = out_events {
        for e in events {
            match e {
                OutEvent::Text(t) => socket.send(Message::Text(t)).unwrap(),
                OutEvent::Event(e) => {
                    let decoded = e.to_binary().unwrap();
                    socket.send(Message::Binary(decoded)).unwrap();
                }
            };
        }
    }
}

pub fn consume_incoming_messages<T: Event>(
    message_bus: ResMut<MessageBus<T>>,
    mut events: EventWriter<T>,
) {
    let messages = { message_bus.lock().split_off(0) };

    for msg in messages {
        events.send(msg);
    }
}

pub fn send_outgoing_messages<T: Event + Clone>(
    message_bus: ResMut<MessageBus<T>>,
    mut events: EventReader<T>,
) {
    let mut messages = message_bus.lock();

    for event in events.read() {
        messages.push(event.clone());
    }
}
