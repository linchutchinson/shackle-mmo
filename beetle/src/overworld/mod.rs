mod player;
mod spawner;

use std::collections::{HashMap, VecDeque};

use client::{Client, ClientEvent};
use common::{math::Vec2, GameObject, NetworkID, PLAY_AREA_SIZE};
use crossbeam_channel::{unbounded, Receiver, Sender};
use legion::{system, systems::CommandBuffer, Entity, Schedule};
use macroquad::{
    prelude::{BLACK, DARKBLUE, DARKBROWN, DARKGRAY},
    shapes::draw_rectangle,
    text::draw_text,
    window::{screen_height, screen_width},
};

use crate::{
    draw_clear_color_system,
    ui::{add_ui_layout_systems, add_ui_rendering_systems},
    ClearColor, Schedules,
};

use self::{
    player::{draw_world_objects_system, move_player_system},
    spawner::{
        spawn_local_player, spawn_overworld_entities_system, spawn_overworld_ui_system,
        spawn_remote_player,
    },
};

pub struct ChatMessageChannel(Sender<String>, Receiver<String>);

pub fn overworld_schedules() -> Schedules {
    let enter_schedule = Schedule::builder()
        .add_system(initialize_overworld_resources_system())
        .flush()
        .add_system(spawn_overworld_ui_system())
        .add_system(spawn_overworld_entities_system())
        .build();

    let mut tick_sbuilder = Schedule::builder();
    tick_sbuilder
        .add_system(handle_client_events_system())
        .flush()
        .add_system(move_player_system())
        .add_system(handle_sending_messages_system());
    add_ui_layout_systems::<()>(&mut tick_sbuilder);
    let tick_schedule = tick_sbuilder.build();

    let mut render_sbuilder = Schedule::builder();
    render_sbuilder
        .add_thread_local(draw_clear_color_system())
        .add_thread_local(draw_play_area_system())
        .add_thread_local(draw_world_objects_system());
    add_ui_rendering_systems::<()>(&mut render_sbuilder);
    render_sbuilder.add_thread_local(draw_chatlog_system());
    let render_schedule = render_sbuilder.build();

    Schedules {
        enter_schedule,
        tick_schedule,
        render_schedule,
    }
}

#[system]
fn handle_sending_messages(
    #[resource] client: &mut Client,
    #[resource] message_stream: &ChatMessageChannel,
) {
    let r = message_stream.1.clone();
    r.try_iter().for_each(|m| {
        client
            .send_chat_message(&m)
            .expect("Just close your eyes and pretend it will always work out.");
    });
}

#[system]
fn draw_chatlog(#[resource] chatlog: &ChatMessages) {
    let screen_height = screen_height();
    chatlog.0.iter().rev().enumerate().for_each(|(idx, s)| {
        let y = screen_height - idx as f32 * 32.0 - 64.0;

        draw_text(&s, 16.0, y, 24.0, BLACK);
    });
}

#[system]
fn draw_play_area() {
    let screen_width = screen_width();
    let screen_height = screen_height();

    let tl = (Vec2::new(screen_width, screen_height) * 0.5) - PLAY_AREA_SIZE * 0.5;

    draw_rectangle(tl.x, tl.y, PLAY_AREA_SIZE.x, PLAY_AREA_SIZE.y, DARKGRAY);
}

const MAX_DISPLAYED_MESSAGES: usize = 5;
struct ChatMessages(VecDeque<String>);

impl ChatMessages {
    fn new() -> Self {
        Self(VecDeque::new())
    }

    fn add_message(&mut self, author: &str, text: &str) {
        let message = format!("{author}: {text}");
        self.0.push_back(message);

        while self.0.len() > MAX_DISPLAYED_MESSAGES {
            self.0.pop_front();
        }
    }
}

#[system]
fn initialize_overworld_resources(commands: &mut CommandBuffer) {
    commands.exec_mut(|_, resources| {
        let clear_color = ClearColor(DARKBROWN);
        resources.insert(clear_color);
        resources.insert(NetworkedEntities(HashMap::new()));
        resources.insert(ChatMessages::new());

        let (s, r) = unbounded();
        resources.insert(ChatMessageChannel(s, r));
    });
}

pub struct Position(Vec2);
pub struct NetworkedEntities(HashMap<NetworkID, Entity>);

#[system]
fn handle_client_events(
    #[resource] networked_entities: &mut NetworkedEntities,
    #[resource] client: &mut Client,
    #[resource] chat_messages: &mut ChatMessages,
    commands: &mut CommandBuffer,
) {
    client.receive_messages().expect("This should succeed.");
    client
        .get_event_receiver()
        .try_iter()
        .for_each(|event| match event {
            ClientEvent::SpawnEntity(id, entity_type) => {
                if let Some(existing) = networked_entities.0.get(&id) {
                    commands.remove(*existing);
                }

                let e = match entity_type {
                    GameObject::ClientPlayer => spawn_local_player(commands),
                    GameObject::RemotePlayer => spawn_remote_player(commands),
                };

                networked_entities.0.insert(id, e);
            }
            ClientEvent::MoveEntity(id, pos) => {
                if let Some(e) = networked_entities.0.get(&id) {
                    commands.add_component(*e, Position(pos));
                } else {
                    log::error!(
                        "Did not have an entity to reposition. Requesting archetype from server..."
                    );
                    // FIXME: Do not pretend there are never network issues.
                    client
                        .request_id_archetype(id.into())
                        .expect("We just pretend there are never network issues.");
                }
            }
            ClientEvent::MessageReceived(author, text) => {
                log::error!("Received Message: {text} from {author}");
                chat_messages.add_message(&author, &text);
            }
        });
}
