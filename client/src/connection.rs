use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use common::messages::{ClientMessage, DisconnectReason, ServerMessage};
use laminar::{Config, ErrorKind, Packet, Socket, SocketEvent};

#[derive(Clone)]
pub enum ConnectionStatus {
    NotConnected,
    Connecting,
    Connected,
    Failed(DisconnectReason),
}

fn client_socket_config() -> Config {
    Config {
        heartbeat_interval: Some(Duration::from_secs(30)),
        ..Default::default()
    }
}

pub struct Connection {
    server_addr: SocketAddr,
    socket: Socket,
}

pub trait ConnectionInterface
where
    Self: Sized,
{
    fn new() -> Result<Self, ErrorKind>;
    fn send_message(&mut self, message: ClientMessage) -> Result<(), ErrorKind>;
    fn receive_messages(&mut self) -> Vec<ServerMessage>;
}

impl ConnectionInterface for Connection {
    fn new() -> Result<Self, ErrorKind> {
        // TODO Select a valid port to bind to in a more sophisticated way.
        let socket = Socket::bind_with_config("0.0.0.0:0", client_socket_config())?;

        // FIXME This is not a real server address.
        let addr_string = std::env::var("SHACKLE_SERVER").unwrap_or("5.78.56.23".to_string());
        let server_addr = format!("{addr_string}:27008").parse().unwrap();
        println!("{server_addr:?}");

        Ok(Self {
            server_addr,
            socket,
        })
    }

    fn send_message(&mut self, message: ClientMessage) -> Result<(), ErrorKind> {
        let payload = message.to_payload();
        let msg_type = MessageType::from(message);
        let packet = match msg_type {
            MessageType::ReliableUnordered => Packet::reliable_unordered(self.server_addr, payload),
            MessageType::Unreliable => Packet::unreliable(self.server_addr, payload),
        };
        self.socket.send(packet)?;
        self.socket.manual_poll(Instant::now());
        Ok(())
    }

    fn receive_messages(&mut self) -> Vec<ServerMessage> {
        let mut result = Vec::new();

        self.socket.manual_poll(Instant::now());

        while let Some(event) = self.socket.recv() {
            if let SocketEvent::Packet(pck) = event {
                let msg_result = ServerMessage::from_payload(pck.payload());

                if let Ok(msg) = msg_result {
                    log::info!("Received Message: {msg:?}");
                    result.push(msg);
                } else {
                    let err = msg_result.unwrap_err();
                    log::error!("Received an invalid packet from the server. {err}");
                }
            }
        }

        result
    }
}

enum MessageType {
    Unreliable,
    ReliableUnordered,
}

impl From<ClientMessage> for MessageType {
    fn from(item: ClientMessage) -> Self {
        match item {
            ClientMessage::MoveTo(_) => Self::Unreliable,
            _ => Self::ReliableUnordered,
        }
    }
}
