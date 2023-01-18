use super::{
    player::{HoverName, NeedsName},
    spawner::{spawn_local_player, spawn_remote_player},
    ChatMessages, NetworkedEntities, OverworldNotifications, Position,
};
use client::{ClientEvent, NetworkClient};
use common::{messages::InfoSendType, GameArchetype};
use legion::{system, systems::CommandBuffer};

#[system]
pub fn handle_client_events(
    #[resource] networked_entities: &mut NetworkedEntities,
    #[resource] client: &mut NetworkClient,
    #[resource] chat_messages: &mut ChatMessages,
    #[resource] notifications: &mut OverworldNotifications,
    commands: &mut CommandBuffer,
) {
    client.receive_messages().expect("This should succeed.");
    client
        .get_event_receiver()
        .try_iter()
        .for_each(|event| match event {
            ClientEvent::SpawnEntity(id, entity_type, is_owned) => {
                if let Some(existing) = networked_entities.0.get(&id) {
                    commands.remove(*existing);
                }

                let e = match entity_type {
                    GameArchetype::Player => {
                        if is_owned {
                            spawn_local_player(commands)
                        } else {
                            spawn_remote_player(commands)
                        }
                    }
                };

                commands.add_component(e, id);
                networked_entities.0.insert(id, e);
            }
            ClientEvent::DespawnEntity(id) => {
                if let Some(e)  = networked_entities.0.remove(&id) {
                    commands.remove(e);
                }
                // We don't mind if this silently passes if the entity wasn't spawned in.
            }
            ClientEvent::UpdateEntityInfo(id, info) => {
                if let Some(e) = networked_entities.0.get(&id) {
                    match info {
                        InfoSendType::Position(pos) => {
                            commands.add_component(*e, Position(pos));
                        }
                        InfoSendType::Identity(name) => {
                            commands.add_component(*e, HoverName { name, radius: 24.0 });
                            commands.remove_component::<NeedsName>(*e);}
                    }
                } else {
                    log::info!(
                        "Did not have an entity to update with given info. Requesting archetype from server..."
                    );
                    // FIXME: Do not pretend there are never network issues.
                    client
                        .request_id_archetype(id)
                        .expect("We just pretend there are never network issues.");
                }
            }
            ClientEvent::MessageReceived(author, text) => {
                log::info!("Received Message: {text} from {author}");
                chat_messages.add_message(&author, &text);
            }
            ClientEvent::ChallengeReceived(sender) => {
                notifications.0.push_back(super::OverworldNotification::ReceivedChallenge(sender));
            }
        });
}
