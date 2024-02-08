use tungstenite::WebSocket;
use bevy::prelude::*;
use std::net::{TcpListener, TcpStream, AddrParseError};
use std::thread;
use thiserror;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("Failed to parse address: {0}")]
    FailedToBincTcpListener(#[from]AddrParseError),
}

#[derive(Resource, Default)]
pub struct MessageBus<T> {
    messages: Vec<T>,
}

fn handle_client(stream: TcpStream) {
    let mut socket = tungstenite::accept(stream).unwrap();

    loop {
        match socket.read() {
            Ok(msg) => println!("{:?}", msg),
            Err(e) => {
                println!("{:?}", e);
                return;
            },
        }
    }
}

pub fn setup_websocket(commands: Commands) -> Result<(), Error> {
    let server = TcpListener::bind("127.0.0.0:420").unwrap();
    let stream = server.accept().unwrap();
    handle_client(stream);

    commands.init_resource::<MessageBus<u32>>();
}

