use client::Client;
use crossbeam_channel::{unbounded, Sender};
use legion::{system, systems::CommandBuffer};

use crate::ui::{
    spawner::{spawn_button, spawn_dynamic_text, spawn_spacer, spawn_ui_container},
    UIRoot, UISize,
};

#[system]
pub fn spawn_overworld_ui(#[resource] client: &Client, commands: &mut CommandBuffer) {
    // TODO: I want to be able to access the user's chosen name from the client. To put in UI

    // FIXME: This is an obviously temporary measure. Replace with an actual event handler.
    let (s, _) = unbounded();

    let top_text = spawn_dynamic_text(commands, "Welcome to Shackle, [USERNAME HERE]");
    let button_spacer = spawn_spacer(commands);
    commands.add_component(button_spacer, UISize::Constant(32.0));
    let logout_button = spawn_button(commands, "Log Out", s, ());

    // FIXME: spawn_spacer should have ui in its function name like other ui spawning functions.
    /*
    let spacer = spawn_spacer(commands);
    */
    let spacer = spawn_dynamic_text(commands, "Pretend there's a really cool game screen here.");
    commands.add_component(spacer, UISize::Grow(10));

    let root_container =
        spawn_ui_container(commands, &[top_text, button_spacer, logout_button, spacer]);
    commands.add_component(root_container, UIRoot);
}

#[system]
pub fn spawn_overworld_entities() {}
