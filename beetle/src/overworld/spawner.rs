use client::Client;
use common::math::Vec2;
use crossbeam_channel::{unbounded, Sender};
use legion::{system, systems::CommandBuffer};

use crate::ui::{
    spawner::{spawn_button, spawn_dynamic_text, spawn_spacer, spawn_ui_container},
    UIRoot, UISize,
};

use super::{player::Player, Position, TILE_SIZE};

#[system]
pub fn spawn_overworld_ui(#[resource] client: &Client, commands: &mut CommandBuffer) {
    // TODO: I want to be able to access the user's chosen name from the client. To put in UI

    // FIXME: This is an obviously temporary measure. Replace with an actual event handler.
    let (s, _) = unbounded();

    let username = client
        .get_username()
        .expect("There should always be a username by this point.");
    let top_text = spawn_dynamic_text(commands, &format!("Welcome to Shackle, {username}"));
    let button_spacer = spawn_spacer(commands);
    commands.add_component(button_spacer, UISize::Constant(32.0));
    let logout_button = spawn_button(commands, "Log Out", s, ());

    // FIXME: spawn_spacer should have ui in its function name like other ui spawning functions.
    let spacer = spawn_spacer(commands);
    commands.add_component(spacer, UISize::Grow(10));

    let root_container =
        spawn_ui_container(commands, &[top_text, button_spacer, logout_button, spacer]);
    commands.add_component(root_container, UIRoot);
}

#[system]
pub fn spawn_overworld_entities(commands: &mut CommandBuffer) {
    commands.push((
        Position(Vec2::new(8.0 * TILE_SIZE, 8.0 * TILE_SIZE)),
        Player,
    ));
}
