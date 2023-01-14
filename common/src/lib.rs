use math::Vec2;
use serde::{Deserialize, Serialize};

pub mod math;
pub mod messages;
pub mod validation;

pub const PLAY_AREA_SIZE: Vec2 = Vec2 { x: 800.0, y: 600.0 };

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
    Player,
}
