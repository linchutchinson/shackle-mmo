use client::{functionality::DuelingClient, NetworkClient};
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
    #[resource] client: &mut NetworkClient,
    #[resource] next_state: &mut NextState,
) {
    ui_event_channel
        .1
        .try_iter()
        .for_each(|event| handle_event(&event, client, next_state));
}

fn handle_event<T: DuelingClient>(
    event: &OverworldUIEvent,
    client: &mut T,
    next_state: &mut NextState,
) {
    match event {
        OverworldUIEvent::Challenge(id) => {
            let result = client.send_challenge(*id);

            if let Err(e) = result {
                log::error!("There was an error sending your duel challenge! {e:?}");
            }
        }
        OverworldUIEvent::Logout => {
            next_state.0 = Some(crate::AppState::MainMenu);
        }
    }
}

#[cfg(test)]
mod tests {
    use client::test_utils::TestClient;
    use common::messages::ClientMessage;
    use crossbeam_channel::unbounded;

    use super::*;

    #[test]
    fn test_logout_event() {
        let (s, r) = unbounded();
        let event_channel = OverworldUIEventChannel(s, r);
        let mut next_state = NextState(None);
        let mut client = TestClient::already_connected();
        event_channel
            .0
            .send(OverworldUIEvent::Logout)
            .expect("Always sends");

        handle_event(&OverworldUIEvent::Logout, &mut client, &mut next_state);

        assert_eq!(next_state.0, Some(crate::AppState::MainMenu))
    }

    #[test]
    fn test_duel_event() {
        let (s, r) = unbounded();
        let event_channel = OverworldUIEventChannel(s, r);
        let mut next_state = NextState(None);
        let mut client = TestClient::already_connected();
        event_channel
            .0
            .send(OverworldUIEvent::Logout)
            .expect("Always sends");

        let target = NetworkID::new(1);
        handle_event(
            &OverworldUIEvent::Challenge(target.clone()),
            &mut client,
            &mut next_state,
        );

        let binding = client.get_sent_messages();
        let last_message = binding.last().unwrap();
        assert_eq!(*last_message, ClientMessage::IssueChallenge(target));
    }
}
