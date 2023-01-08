use legion::{system, systems::CommandBuffer};

use crate::{
    main_menu::event::MainMenuEvent,
    ui::{
        spawner::{
            spawn_button, spawn_dynamic_text, spawn_spacer, spawn_text_input, spawn_ui_container,
        },
        UIConstraint, UIRoot, UISize,
    },
};

use super::event::MainMenuEventHandler;

pub fn spawn_login_menu(commands: &mut CommandBuffer, event_handler: &MainMenuEventHandler) {
    log::trace!("Spawning Login Menu Components...");

    let title_text = spawn_dynamic_text(commands, "Enter Your Name");
    commands.add_component(title_text, UISize::Grow(1));
    commands.add_component(title_text, UIConstraint::width_constraint(512.0));

    let spacer = spawn_spacer(commands);
    commands.add_component(spacer, UISize::Constant(32.0));

    let button_spacer_1 = spawn_spacer(commands);
    commands.add_component(button_spacer_1, UISize::Grow(1));

    let username_input = spawn_text_input(commands);

    let login_button = spawn_button(
        commands,
        "Log In",
        event_handler.1.clone(),
        MainMenuEvent::LoginButtonClicked(username_input),
    );

    let button_spacer_2 = spawn_spacer(commands);
    commands.add_component(button_spacer_2, UISize::Grow(3));

    let button_container = spawn_ui_container(
        commands,
        &[
            button_spacer_1,
            username_input,
            login_button,
            button_spacer_2,
        ],
    );
    commands.add_component(button_container, UISize::Grow(5));

    let root = spawn_ui_container(commands, &[title_text, spacer, button_container]);
    commands.add_component(root, UIRoot);
}

#[system]
pub fn spawn_main_menu(
    commands: &mut CommandBuffer,
    #[resource] event_handler: &MainMenuEventHandler,
) {
    log::trace!("Spawning Main Menu Components...");

    let title_text = spawn_dynamic_text(commands, "Shackle MMO");
    commands.add_component(title_text, UISize::Grow(1));
    commands.add_component(title_text, UIConstraint::width_constraint(512.0));

    let spacer = spawn_spacer(commands);
    commands.add_component(spacer, UISize::Constant(32.0));

    let button_spacer_1 = spawn_spacer(commands);
    commands.add_component(button_spacer_1, UISize::Grow(1));

    let play_button = spawn_button(
        commands,
        "Play",
        event_handler.1.clone(),
        MainMenuEvent::PlayButtonClicked,
    );

    let quit_button = spawn_button(
        commands,
        "Quit",
        event_handler.1.clone(),
        MainMenuEvent::QuitButtonClicked,
    );

    let button_spacer_2 = spawn_spacer(commands);
    commands.add_component(button_spacer_2, UISize::Grow(3));

    let button_container = spawn_ui_container(
        commands,
        &[button_spacer_1, play_button, quit_button, button_spacer_2],
    );
    commands.add_component(button_container, UISize::Grow(5));

    let root = spawn_ui_container(commands, &[title_text, spacer, button_container]);
    commands.add_component(root, UIRoot);
}
