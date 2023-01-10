use std::net::SocketAddr;

use common::ClientMessage;
use laminar::{ErrorKind, Packet, Socket, SocketEvent};

pub struct Client {
    connection: Option<Connection>,
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
        self.connection = Some(conn);

        let result = self
            .connection
            .as_mut()
            .unwrap()
            .send_message(ClientMessage::Connect(username.to_string()));

        if result.is_err() {
            return Err(ClientError::NetworkError(result.unwrap_err()));
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ClientError {
    DuplicateConnectionError,
    NetworkError(ErrorKind),
}

enum ConnectionStatus {
    Connecting,
    Connected,
    Failed(()),
}

struct Connection {
    server_addr: SocketAddr,
    status: ConnectionStatus,
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
            status: ConnectionStatus::Connecting,
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
}
