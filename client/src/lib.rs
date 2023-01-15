mod connection;
mod dueling;
pub mod functionality;
pub use connection::ConnectionStatus;
use connection::{Connection, ConnectionInterface};

use common::{
    math::Vec2,
    messages::{ClientMessage, InfoRequestType, InfoSendType, ServerMessage},
    GameArchetype, NetworkID,
};
use crossbeam_channel::{unbounded, Receiver, Sender};
use laminar::ErrorKind;

pub type NetworkClient = Client<Connection>;

pub struct Client<T: ConnectionInterface> {
    connection: Option<(T, ConnectionStatus)>,
    username: Option<String>,
    sender: Sender<ClientEvent>,
    receiver: Receiver<ClientEvent>,
}

impl<T: ConnectionInterface> Default for Client<T> {
    fn default() -> Self {
        let (sender, receiver) = unbounded();
        Self {
            connection: None,
            username: None,
            sender,
            receiver,
        }
    }
}

impl<T: ConnectionInterface> Client<T> {
    pub fn connect(&mut self, username: &str) -> Result<(), ClientError> {
        if self.connection.is_some() {
            return Err(ClientError::DuplicateConnectionError);
        }

        let conn = T::new()?;
        self.connection = Some((conn, ConnectionStatus::Connecting));

        let result = self
            .connection
            .as_mut()
            .unwrap()
            .0
            .send_message(ClientMessage::Connect(username.to_string()));

        if let Err(err) = result {
            return Err(ClientError::NetworkError(err));
        }

        self.username = Some(username.to_string());

        Ok(())
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        if self.connection.is_none() {
            ConnectionStatus::NotConnected
        } else {
            let conn = self.connection.as_ref().unwrap();
            conn.1.clone()
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
                conn.1 = ConnectionStatus::Failed(*reason);
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
                self.sender
                    .send(ClientEvent::ChallengeReceived(*sender))
                    .expect("This should send.");
            }
            ServerMessage::ChangeClientMode(_new_mode) => {
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

    fn get_connection_mut(&mut self) -> Result<&mut T, ClientError> {
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
    ChallengeReceived(NetworkID),
}

#[cfg(feature = "test_client")]
pub mod test_utils {
    use super::*;

    pub type TestClient = Client<TestConnection>;

    impl TestClient {
        pub fn already_connected() -> Self {
            let mut result = Self::default();
            result.connect("TestUser").expect("This always works.");
            result
        }

        pub fn get_sent_messages(&mut self) -> Vec<ClientMessage> {
            let conn = self.get_connection_mut().expect("Client always exists.");
            conn.client_message_channel.1.try_iter().collect()
        }
    }

    pub struct TestConnection {
        client_message_channel: (Sender<ClientMessage>, Receiver<ClientMessage>),
        server_message_channel: (Sender<ServerMessage>, Receiver<ServerMessage>),
    }

    /*
    impl TestConnection {
        fn fake_server_message(&self, msg: ServerMessage) {
            self.server_message_channel
                .0
                .send(msg)
                .expect("This always works.");
        }
    }
    */

    impl ConnectionInterface for TestConnection {
        fn new() -> Result<Self, ErrorKind> {
            let client_message_channel = unbounded();
            let server_message_channel = unbounded();
            Ok(Self {
                client_message_channel,
                server_message_channel,
            })
        }

        fn send_message(&mut self, message: ClientMessage) -> Result<(), ErrorKind> {
            self.client_message_channel
                .0
                .send(message)
                .expect("This always sends");
            Ok(())
        }

        fn receive_messages(&mut self) -> Vec<ServerMessage> {
            self.server_message_channel.1.try_iter().collect()
        }
    }
}

#[cfg(test)]
mod test {}
