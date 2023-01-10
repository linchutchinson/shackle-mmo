use std::{net::SocketAddr, time::Instant};

use common::{ClientMessage, DisconnectReason, ServerMessage};
use laminar::{ErrorKind, Packet, Socket, SocketEvent};

pub struct Client {
    connection: Option<(Connection, ConnectionStatus)>,
}

impl Client {
    pub fn new() -> Self {
        Self { connection: None }
    }

    pub fn connect(&mut self, username: &str) -> Result<(), ClientError> {
        if self.connection.is_some() {
            return Err(ClientError::DuplicateConnectionError);
        }

        let conn = Connection::new().unwrap();
        self.connection = Some((conn, ConnectionStatus::Connecting));

        let result = self
            .connection
            .as_mut()
            .unwrap()
            .0
            .send_message(ClientMessage::Connect(username.to_string()));

        if result.is_err() {
            return Err(ClientError::NetworkError(result.unwrap_err()));
        }

        Ok(())
    }

    pub fn receive_messages(&mut self) -> Result<(), ClientError> {
        if self.connection.is_none() {
            return Err(ClientError::NotConnected);
        }

        let conn = self.connection.as_mut().unwrap();
        let messages = conn.0.receive_messages();

        messages.iter().for_each(|msg| match msg {
            ServerMessage::ConnectionAccepted => conn.1 = ConnectionStatus::Connected,
            ServerMessage::DisconnectClient(reason) => {
                conn.1 = ConnectionStatus::Failed(reason.clone())
            }
        });

        Ok(())
    }
}

#[derive(Debug)]
pub enum ClientError {
    DuplicateConnectionError,
    NetworkError(ErrorKind),
    NotConnected,
}

enum ConnectionStatus {
    Connecting,
    Connected,
    Failed(DisconnectReason),
}

struct Connection {
    server_addr: SocketAddr,
    socket: Socket,
}

impl Connection {
    fn new() -> Result<Self, ErrorKind> {
        // TODO Select a valid port to bind to in a more sophisticated way.
        let socket = Socket::bind("127.0.0.1:12351")?;

        // FIXME This is not a real server address.
        let server_addr = "127.0.0.1:12352".parse().unwrap();

        Ok(Self {
            server_addr,
            socket,
        })
    }

    fn send_message(&mut self, message: ClientMessage) -> Result<(), ErrorKind> {
        println!("It's not about the gameplay,");
        println!("It's about sending a message.");
        let payload = message.to_payload();
        self.socket
            .send(Packet::reliable_unordered(self.server_addr, payload))?;
        Ok(())
    }

    fn receive_messages(&mut self) -> Vec<ServerMessage> {
        let mut result = Vec::new();

        self.socket.manual_poll(Instant::now());

        while let Some(event) = self.socket.recv() {
            match event {
                SocketEvent::Packet(pck) => {
                    let msg_result = ServerMessage::from_payload(pck.payload().into());

                    if let Ok(msg) = msg_result {
                        result.push(msg);
                    } else {
                        let err = msg_result.unwrap_err();
                        log::error!("Received an invalid packet from the server. {err}");
                    }
                }
                _ => {}
            }
        }

        result
    }
}
