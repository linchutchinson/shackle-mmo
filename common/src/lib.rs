use serde::{Deserialize, Serialize};

pub mod math;
pub mod validation;

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    // The String is the desired Username
    Connect(String),
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
