use std::{
    net::SocketAddr,
    time::{Duration, Instant},
};

use common::{
    math::Vec2, ClientMessage, DisconnectReason, GameArchetype, InfoRequestType, InfoSendType,
    NetworkID, ServerMessage,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use laminar::{Config, ErrorKind, Packet, Socket, SocketEvent};

pub struct Client {
    connection: Option<(Connection, ConnectionStatus)>,
    username: Option<String>,
    sender: Sender<ClientEvent>,
    receiver: Receiver<ClientEvent>,
}

impl Client {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            connection: None,
            username: None,
            sender,
            receiver,
        }
    }

    pub fn connect(&mut self, username: &str) -> Result<(), ClientError> {
        if self.connection.is_some() {
            return Err(ClientError::DuplicateConnectionError);
        }

        let conn = Connection::new()?;
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

        self.username = Some(username.to_string());

        Ok(())
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        if self.connection.is_none() {
            return ConnectionStatus::NotConnected;
        } else {
            let conn = self.connection.as_ref().unwrap();
            return conn.1.clone();
        }
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
                self.username = None;
                conn.1 = ConnectionStatus::Failed(reason.clone());
            }
            ServerMessage::SpawnNetworkedEntity(id, entity_type, is_owned) => {
                self.sender
                    .send(ClientEvent::SpawnEntity(*id, *entity_type, *is_owned))
                    .expect("This should send.");
            }
            ServerMessage::DespawnNetworkedEntity(id) => {
                self.sender
                    .send(ClientEvent::DespawnEntity(*id))
                    .expect("This should send.");
            }
            ServerMessage::SendNetworkedEntityInfo(id, info) => {
                self.sender
                    .send(ClientEvent::UpdateEntityInfo(*id, info.clone()))
                    .expect("This should send.");
            }
            ServerMessage::SendMessage(author, text) => {
                self.sender
                    .send(ClientEvent::MessageReceived(
                        author.to_string(),
                        text.to_string(),
                    ))
                    .expect("This should send.");
            }
        });

        Ok(())
    }

    pub fn get_username(&self) -> Option<&str> {
        match &self.username {
            Some(s) => Some(s.as_ref()),
            None => None,
        }
    }

    pub fn get_event_receiver(&self) -> Receiver<ClientEvent> {
        self.receiver.clone()
    }

    fn get_connection_mut(&mut self) -> Result<&mut Connection, ClientError> {
        if self.connection.is_none() {
            return Err(ClientError::NotConnected);
        }

        Ok(&mut self.connection.as_mut().unwrap().0)
    }

    pub fn move_player(&mut self, pos: Vec2) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;

        conn.send_message(ClientMessage::MoveTo(pos))?;
        Ok(())
    }

    pub fn request_id_archetype(&mut self, id: NetworkID) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;
        conn.send_message(ClientMessage::RequestArchetype(id))?;
        Ok(())
    }

    pub fn request_id_info(
        &mut self,
        id: NetworkID,
        info: InfoRequestType,
    ) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;
        conn.send_message(ClientMessage::RequestEntityInfo(id, info))?;
        Ok(())
    }

    pub fn send_chat_message(&mut self, text: &str) -> Result<(), ClientError> {
        let conn = self.get_connection_mut()?;
        conn.send_message(ClientMessage::SendMessage(text.to_string()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ClientError {
    DuplicateConnectionError,
    NetworkError(ErrorKind),
    NotConnected,
}

impl From<ErrorKind> for ClientError {
    fn from(source: ErrorKind) -> Self {
        Self::NetworkError(source)
    }
}

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

struct Connection {
    server_addr: SocketAddr,
    socket: Socket,
}

impl Connection {
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
            match event {
                SocketEvent::Packet(pck) => {
                    let msg_result = ServerMessage::from_payload(pck.payload().into());

                    if let Ok(msg) = msg_result {
                        log::info!("Received Message: {msg:?}");
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

pub enum ClientEvent {
    SpawnEntity(NetworkID, GameArchetype, bool),
    DespawnEntity(NetworkID),
    UpdateEntityInfo(NetworkID, InfoSendType),
    MessageReceived(String, String),
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
