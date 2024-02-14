use bevy::prelude::*;
use serde::Deserialize;
use std::net::{AddrParseError, TcpListener, TcpStream};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use std::thread;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse address: {0}")]
    FailedToBincTcpListener(#[from] AddrParseError),
}

#[derive(Resource, Default, Debug)]
pub struct MessageBus<T: Send> {
    messages: Arc<Mutex<Vec<T>>>,
}

impl<T: Send> Clone for MessageBus<T> {
    fn clone(&self) -> Self {
        Self {
            messages: self.messages.clone(),
        }
    }
}

impl<'a, T: Send> MessageBus<T> {
    pub fn new() -> Self {
        MessageBus {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn lock(&mut self) -> LockResult<MutexGuard<'_, Vec<T>>> {
        self.messages.lock()
    }
}

pub fn setup_websocket<'a, T: Send + 'static>(mut commands: Commands) -> Result<(), Error>
where
    for<'de> T: Deserialize<'de> + 'a,
{
    let message_bus = MessageBus::<T>::new();
    let server = TcpListener::bind("localhost:8876").unwrap();

    let mb = message_bus.clone();
    thread::spawn(move || handle_connections(server, mb));

    commands.init_resource::<MessageBus<u32>>();
    commands.insert_resource(message_bus);
    Ok(())
}

fn handle_connections<'a, T: Send + 'static>(server: TcpListener, message_bus: MessageBus<T>)
where
    for<'de> T: Deserialize<'de> + 'a,
{
    for stream in server.incoming() {
        let mb = message_bus.clone();
        thread::spawn(move || match stream {
            Ok(s) => handle_client(s, mb),
            Err(e) => println!("{:?}", e),
        });
    }
}

fn handle_client<'a, T: Send>(stream: TcpStream, mut message_bus: MessageBus<T>)
where
    for<'de> T: Deserialize<'de> + 'a,
{
    let mut socket = tungstenite::accept(stream).unwrap();
    loop {
        match socket.read() {
            Ok(msg) => match msg {
                tungstenite::Message::Text(json) => {
                    let decoded: T = serde_json::from_str(&json).unwrap();
                    let mut msg_bus = message_bus.lock().unwrap();
                    msg_bus.push(decoded);
                    println!("len: {}", msg_bus.len())
                }
                tungstenite::Message::Binary(_) => todo!(),
                tungstenite::Message::Ping(_) => todo!(),
                tungstenite::Message::Pong(_) => todo!(),
                tungstenite::Message::Close(_) => todo!(),
                tungstenite::Message::Frame(_) => todo!(),
            },
            Err(_) => todo!(),
        }
    }
}
