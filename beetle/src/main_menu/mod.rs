mod event;
mod spawner;

use client::{ConnectionStatus, NetworkClient};
use common::validation::validate_username;
use legion::{
    system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query, Schedule,
};
use log::{error, info};
use macroquad::prelude::DARKBLUE;

use crate::{
    draw_clear_color_system,
    ui::{add_ui_layout_systems, add_ui_rendering_systems, Text},
    ClearColor, NextState, Schedules,
};

use self::{
    event::{MainMenuEvent, MainMenuEventHandler, MainMenuNotification, NotificationDisplay},
    spawner::{spawn_connecting_screen, spawn_login_menu, spawn_main_menu_system},
};

pub fn main_menu_schedules() -> Schedules {
    let enter_schedule = Schedule::builder()
        .add_system(init_main_menu_resources_system())
        .flush()
        .add_system(spawn_main_menu_system())
        .build();

    let mut tick_schedule_builder = Schedule::builder();
    add_ui_layout_systems::<MainMenuEvent>(&mut tick_schedule_builder);
    let tick_schedule = tick_schedule_builder
        .add_system(handle_main_menu_events_system())
        .add_system(display_notifications_system())
        .add_system(join_server_when_connected_system())
        .build();

    let render_schedule = render_schedule();

    Schedules {
        enter_schedule,
        tick_schedule,
        render_schedule,
    }
}

fn render_schedule() -> Schedule {
    let mut builder = Schedule::builder();
    builder.add_thread_local(draw_clear_color_system());

    add_ui_rendering_systems::<MainMenuEvent>(&mut builder);

    builder.build()
}

#[system(for_each)]
fn display_notifications(notifications: &mut NotificationDisplay, text: &mut Text) {
    notifications
        .0
        .try_iter()
        .for_each(|notification| match notification {
            MainMenuNotification::Error(msg) => {
                println!("{}", msg);
                text.0 = msg;
            }
        });
}

#[system]
#[read_component(Text)]
fn handle_main_menu_events(
    world: &mut SubWorld,
    query: &mut Query<Entity>,
    #[resource] handler: &mut MainMenuEventHandler,
    #[resource] next_state: &mut NextState,
    #[resource] client: &mut NetworkClient,
    commands: &mut CommandBuffer,
) {
    let receiver = handler.event_receiver().clone();
    receiver.try_iter().for_each(|event| match event {
        MainMenuEvent::PlayButtonClicked => {
            // FIXME Currently this clears the current UI by removing everything. This seems prone to problems.
            // So I'm going to think about it, as there's probably a more appropriate solution. Having a tag component for current
            // UI elements feels clunky, but that may be due to the way I created the spawner code.
            query.iter(world).for_each(|e| {
                commands.remove(*e);
            });
            spawn_login_menu(commands, handler);
        }
        MainMenuEvent::QuitButtonClicked => {
            next_state.0 = Some(crate::AppState::Quit);
        }
        MainMenuEvent::LoginButtonClicked(input_entity) => {
            let entry = world
                .entry_ref(input_entity)
                .expect("The login button was not connected to an existing entity.");

            let text = entry
                .get_component::<Text>()
                .expect("The login button was connected to an entity without text.");

            let validity_check = validate_username(&text.0);

            if validity_check.is_ok() {
                // Log In
                info!("Logging in with username: {}", text.0);
                let connection_result = client.connect(&text.0);

                match connection_result {
                    Ok(()) => {
                        query.iter(world).for_each(|e| {
                            commands.remove(*e);
                        });

                        spawn_connecting_screen(commands);
                    }
                    Err(err) => {
                        // TODO: This should use display for user facing formatting instead of debug.
                        let err_msg = format!("{:?}", err);
                        handler.send_notification(MainMenuNotification::Error(err_msg));
                    }
                }
            } else {
                // Error Occurred...
                let err_msg = format!("{}", validity_check.as_ref().unwrap_err());
                error!("{}", validity_check.unwrap_err());
                handler.send_notification(MainMenuNotification::Error(err_msg));
            }
        }
    });
}

#[system]
fn init_main_menu_resources(commands: &mut CommandBuffer) {
    commands.exec_mut(move |_, resources| {
        let event_handler = MainMenuEventHandler::new();
        resources.insert(event_handler);
        resources.insert(ClearColor(DARKBLUE));
        resources.insert(NetworkClient::new());
    });
}

#[system]
fn join_server_when_connected(
    query: &mut Query<Entity>,
    world: &mut SubWorld,
    #[resource] client: &mut NetworkClient,
    #[resource] next_state: &mut NextState,
    commands: &mut CommandBuffer,
) {
    // We don't care at this point whether or not the client is connected.
    // So this result can safely be ignored.
    client.receive_messages().ok();

    match client.connection_status() {
        ConnectionStatus::Connected => {
            // FIXME: Clearing all entities here is a weird temporary measure.
            query.iter(world).for_each(|e| {
                commands.remove(*e);
            });

            next_state.0 = Some(crate::AppState::Overworld);
        }
        _ => {}
    }
}
