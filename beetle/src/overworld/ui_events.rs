use common::NetworkID;
use crossbeam_channel::{Receiver, Sender};
use legion::system;

use crate::NextState;

pub struct OverworldUIEventChannel(pub Sender<OverworldUIEvent>, pub Receiver<OverworldUIEvent>);

#[derive(Copy, Clone)]
pub enum OverworldUIEvent {
    Challenge(NetworkID),
    Logout,
}

#[system]
pub fn handle_overworld_ui_events(
    #[resource] ui_event_channel: &OverworldUIEventChannel,
    #[resource] next_state: &mut NextState,
) {
    ui_event_channel.1.try_iter().for_each(|event| match event {
        OverworldUIEvent::Challenge(_id) => {
            unimplemented!()
        }
        OverworldUIEvent::Logout => {
            next_state.0 = Some(crate::AppState::MainMenu);
        }
    });
}
