use tungstenite::WebSocket;
use bevy::prelude::*;
use std::net::{TcpListener, TcpStream, AddrParseError, SocketAddr};
use std::thread;
use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to parse address: {0}")]
    FailedToBincTcpListener(#[from]AddrParseError),
}

#[derive(Resource, Default)]
pub struct MessageBus<T> {
    messages: Vec<T>,
}

fn handle_client(server: TcpListener) {
    for stream in server.incoming() {
        thread::spawn(move || match stream {
            Ok(s) => {
                let mut socket = tungstenite::accept(s).unwrap();
                loop {
                    let msg = socket.read();
                    println!("{:?}", msg);
                }
            },
            Err(e) => println!("{:?}", e),
        });
    }
}

pub fn setup_websocket(mut commands: Commands) -> Result<(), Error> {
    let server = TcpListener::bind("localhost:8876").unwrap();
    thread::spawn(|| handle_client(server));

    commands.init_resource::<MessageBus<u32>>();
    Ok(())
}

