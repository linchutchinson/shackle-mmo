mod event;
mod spawner;

use client::Client;
use common::{math::Rect, validation::validate_username};
use legion::{
    system, systems::CommandBuffer, world::SubWorld, Entity, EntityStore, Query, Schedule,
};
use log::{error, info, warn};
use macroquad::{prelude::DARKBLUE, window::clear_background};

use crate::{
    ui::{add_ui_layout_systems, add_ui_rendering_systems, Text},
    ClearColor, NextState, Schedules,
};

use self::{
    event::{MainMenuEvent, MainMenuEventHandler, MainMenuNotification, NotificationDisplay},
    spawner::{spawn_login_menu, spawn_main_menu_system},
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
    builder.add_thread_local(draw_main_menu_system());

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
    query: &mut Query<(Entity, &Rect)>,
    #[resource] handler: &mut MainMenuEventHandler,
    #[resource] next_state: &mut NextState,
    #[resource] client: &mut Client,
    commands: &mut CommandBuffer,
) {
    let receiver = handler.event_receiver().clone();
    receiver.try_iter().for_each(|event| match event {
        MainMenuEvent::PlayButtonClicked => {
            // FIXME Currently this clears the current UI by removing everything with a Rect component. This seems prone to problems.
            // So I'm going to think about it, as there's probably a more appropriate solution. Having a tag component for current
            // UI elements feels clunky, but that may be due to the way I created the spawner code.
            query.iter(world).for_each(|(e, _)| {
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
                if connection_result.is_err() {
                    let err = connection_result.unwrap_err();
                    // TODO: This should use display for user facing formatting instead of debug.
                    let err_msg = format!("{:?}", err);
                    handler.send_notification(MainMenuNotification::Error(err_msg));
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
        resources.insert(Client::new());
    });
}

#[system]
fn draw_main_menu(#[resource] clear_color: &ClearColor) {
    clear_background(clear_color.0);
}
