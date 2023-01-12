use math::Vec2;
use serde::{Deserialize, Serialize};

pub mod math;
pub mod validation;

pub const PLAY_AREA_SIZE: Vec2 = Vec2 { x: 800.0, y: 600.0 };

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    // The String is the desired Username
    Connect(String),
    MoveTo(Vec2),
    RequestArchetype(usize),
    SendMessage(String),
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
    SpawnNetworkedEntity(NetworkID, GameArchetype),
    RepositionNetworkedEntity(NetworkID, Vec2),
    SendMessage(String, String),
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum DisconnectReason {
    InvalidUsername,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct NetworkID(usize);

impl NetworkID {
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

impl From<NetworkID> for usize {
    fn from(item: NetworkID) -> Self {
        item.0
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum GameArchetype {
    ClientPlayer,
    RemotePlayer,
}
