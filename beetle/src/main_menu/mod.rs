mod event;
mod spawner;

use common::math::Rect;
use crossbeam_channel::unbounded;
use legion::{system, systems::CommandBuffer, world::SubWorld, Entity, Query, Schedule};
use macroquad::{prelude::DARKBLUE, window::clear_background};

use crate::{
    ui::{add_ui_layout_systems, add_ui_rendering_systems},
    ClearColor, NextState, Schedules,
};

use self::{
    event::{MainMenuEvent, MainMenuEventHandler},
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

#[system]
fn handle_main_menu_events(
    world: &mut SubWorld,
    query: &mut Query<(Entity, &Rect)>,
    #[resource] handler: &mut MainMenuEventHandler,
    #[resource] next_state: &mut NextState,
    commands: &mut CommandBuffer,
) {
    let receiver = &handler.0;
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
    });
}

#[system]
fn init_main_menu_resources(commands: &mut CommandBuffer) {
    let (sender, receiver) = unbounded();

    commands.exec_mut(move |_, resources| {
        let event_handler = MainMenuEventHandler(receiver.clone(), sender.clone());
        resources.insert(event_handler);
        resources.insert(ClearColor(DARKBLUE));
    });
}

#[system]
fn draw_main_menu(#[resource] clear_color: &ClearColor) {
    clear_background(clear_color.0);
}
