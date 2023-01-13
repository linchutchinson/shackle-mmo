use client::Client;
use common::PLAY_AREA_SIZE;
use crossbeam_channel::unbounded;
use legion::{system, systems::CommandBuffer, Entity};
use macroquad::prelude::{GREEN, WHITE};

use crate::ui::{
    spawner::{
        spawn_button, spawn_context_menu, spawn_dynamic_text, spawn_spacer, spawn_text_input,
        spawn_ui_container,
    },
    FullscreenRoot, SubmitOnEnter, UIRoot, UISize,
};

use super::{
    player::{Controller, HoverName, NeedsName, OtherPlayer, Player, WorldDisplay},
    ChatMessageChannel, OverworldUIEvent, OverworldUIEventChannel, Position,
};

#[system]
pub fn spawn_overworld_ui(
    #[resource] client: &Client,
    #[resource] chat_message_channel: &ChatMessageChannel,
    #[resource] ui_event_channel: &OverworldUIEventChannel,
    commands: &mut CommandBuffer,
) {
    let username = client
        .get_username()
        .expect("There should always be a username by this point.");
    let top_text = spawn_dynamic_text(commands, &format!("Welcome to Shackle, {username}"));
    let button_spacer = spawn_spacer(commands);
    commands.add_component(button_spacer, UISize::Constant(32.0));
    let logout_button = spawn_button(
        commands,
        "Log Out",
        ui_event_channel.0.clone(),
        OverworldUIEvent::Logout,
    );

    // FIXME: spawn_spacer should have ui in its function name like other ui spawning functions.
    let spacer = spawn_spacer(commands);
    commands.add_component(spacer, UISize::Grow(10));

    let chat_input = spawn_text_input(commands);
    commands.add_component(chat_input, UISize::Constant(32.0));

    let sender = chat_message_channel.0.clone();
    commands.add_component(chat_input, SubmitOnEnter(sender));

    let root_container = spawn_ui_container(
        commands,
        &[top_text, button_spacer, logout_button, spacer, chat_input],
    );
    commands.add_component(root_container, UIRoot);
    commands.add_component(root_container, FullscreenRoot);
}

#[system]
pub fn spawn_overworld_entities(_commands: &mut CommandBuffer) {}

pub fn spawn_local_player(commands: &mut CommandBuffer) -> Entity {
    let pos = PLAY_AREA_SIZE * 0.5;
    commands.push((
        Position(pos),
        Player,
        Controller,
        WorldDisplay("@".to_string(), WHITE),
        HoverName {
            name: "Me".to_string(),
            radius: 24.0,
        },
    ))
}

pub fn spawn_remote_player(commands: &mut CommandBuffer) -> Entity {
    let pos = PLAY_AREA_SIZE * 0.5;
    commands.push((
        Position(pos),
        Player,
        WorldDisplay("@".to_string(), GREEN),
        NeedsName,
        OtherPlayer,
    ))
}
