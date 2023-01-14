mod connection;
mod dueling;
use connection::Connection;
pub use connection::ConnectionStatus;

use common::{
    math::Vec2,
    messages::{ClientMessage, InfoRequestType, InfoSendType, ServerMessage},
    GameArchetype, NetworkID,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use laminar::ErrorKind;

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
            ServerMessage::PassAlongChallenge(sender) => {
                unimplemented!()
            }
            ServerMessage::ChangeClientMode(new_mode) => {
                unimplemented!()
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

pub enum ClientEvent {
    SpawnEntity(NetworkID, GameArchetype, bool),
    DespawnEntity(NetworkID),
    UpdateEntityInfo(NetworkID, InfoSendType),
    MessageReceived(String, String),
}
