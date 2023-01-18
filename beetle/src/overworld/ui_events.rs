use client::{functionality::DuelingClient, NetworkClient};
use common::NetworkID;
use crossbeam_channel::{Receiver, Sender};
use legion::system;

use crate::NextState;

use super::OverworldNotifications;

pub struct OverworldUIEventChannel(pub Sender<OverworldUIEvent>, pub Receiver<OverworldUIEvent>);

#[derive(Copy, Clone)]
pub enum OverworldUIEvent {
    Challenge(NetworkID),
    ChallengeResponse(NetworkID, bool),
    Logout,
}

#[system]
pub fn handle_overworld_ui_events(
    #[resource] ui_event_channel: &OverworldUIEventChannel,
    #[resource] client: &mut NetworkClient,
    #[resource] notifications: &mut OverworldNotifications,
    #[resource] next_state: &mut NextState,
) {
    ui_event_channel
        .1
        .try_iter()
        .for_each(|event| handle_event(&event, client, next_state, notifications));
}

fn handle_event<T: DuelingClient>(
    event: &OverworldUIEvent,
    client: &mut T,
    next_state: &mut NextState,
    notifications: &mut OverworldNotifications,
) {
    match event {
        OverworldUIEvent::Challenge(id) => {
            let result = client.send_challenge(*id);

            if let Err(e) = result {
                log::error!("There was an error sending your duel challenge! {e:?}");
            }
        }
        OverworldUIEvent::ChallengeResponse(id, response) => {
            let result = client.respond_to_challenge(*id, *response);

            if let Err(e) = result {
                log::error!("There was an error sending your duel response! {e:?}");
            }

            notifications.0.pop_front();
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
        let mut notifications = OverworldNotifications::default();

        handle_event(
            &OverworldUIEvent::Logout,
            &mut client,
            &mut next_state,
            &mut notifications,
        );

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
        let mut notifications = OverworldNotifications::default();

        let target = NetworkID::new(1);
        handle_event(
            &OverworldUIEvent::Challenge(target.clone()),
            &mut client,
            &mut next_state,
            &mut notifications,
        );

        let binding = client.get_sent_messages();
        let last_message = binding.last().unwrap();
        assert_eq!(*last_message, ClientMessage::IssueChallenge(target));
    }
}
