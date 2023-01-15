use serde::{Deserialize, Serialize};

use crate::{math::Vec2, ClientMode, GameArchetype, NetworkID};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ClientMessage {
    // The String is the desired Username
    Connect(String),
    RequestArchetype(NetworkID),
    RequestEntityInfo(NetworkID, InfoRequestType),
    SendMessage(String),
    MoveTo(Vec2),
    IssueChallenge(NetworkID),
    RespondToChallenge(NetworkID, bool),
    Disconnect,
}

impl ClientMessage {
    pub fn to_payload(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    pub fn from_payload(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice::<Self>(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    ConnectionAccepted,
    SpawnNetworkedEntity(NetworkID, GameArchetype, bool),
    DespawnNetworkedEntity(NetworkID),
    SendNetworkedEntityInfo(NetworkID, InfoSendType),
    SendMessage(String, String),
    PassAlongChallenge(NetworkID),
    ChangeClientMode(ClientMode),
    DisconnectClient(DisconnectReason),
}

impl ServerMessage {
    pub fn to_payload(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }

    pub fn from_payload(bytes: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice::<Self>(bytes)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum InfoRequestType {
    Identity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InfoSendType {
    Identity(String),
    Position(Vec2),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum DisconnectReason {
    InvalidUsername,
}
