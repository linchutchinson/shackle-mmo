use legion::{system, systems::CommandBuffer, Schedule};
use macroquad::{prelude::DARKBLUE, window::clear_background};

use crate::{
    ui::{
        add_ui_layout_systems, add_ui_rendering_systems,
        spawner::{spawn_dynamic_text, spawn_spacer, spawn_ui_container},
        UIConstraint, UIRoot, UISize,
    },
    ClearColor, Schedules,
};

pub fn main_menu_schedules() -> Schedules {
    let enter_schedule = Schedule::builder()
        .add_system(set_bg_color_system())
        .add_system(spawn_main_menu_system())
        .build();

    let mut tick_schedule_builder = Schedule::builder();
    add_ui_layout_systems(&mut tick_schedule_builder);
    let tick_schedule = tick_schedule_builder.build();

    let render_schedule = render_schedule();

    Schedules {
        enter_schedule,
        tick_schedule,
        render_schedule,
    }
}

#[system]
fn set_bg_color(#[resource] color: &mut ClearColor) {
    color.0 = DARKBLUE;
}

#[system]
fn spawn_main_menu(commands: &mut CommandBuffer) {
    println!("Spawning Main Menu Components...");
    let title_text = spawn_dynamic_text(commands, "Shackle MMO");
    commands.add_component(title_text, UISize::Grow(1));
    commands.add_component(title_text, UIConstraint::width_constraint(512.0));

    let spacer = spawn_spacer(commands);

    let button_container = spawn_ui_container(commands, &[]);
    commands.add_component(button_container, UISize::Grow(5));

    let root = spawn_ui_container(commands, &[title_text, spacer, button_container]);
    commands.add_component(root, UIRoot);
}

fn render_schedule() -> Schedule {
    let mut builder = Schedule::builder();
    builder.add_thread_local(draw_main_menu_system());

    add_ui_rendering_systems(&mut builder);

    builder.build()
}

#[system]
fn draw_main_menu(#[resource] clear_color: &ClearColor) {
    clear_background(clear_color.0);
}
