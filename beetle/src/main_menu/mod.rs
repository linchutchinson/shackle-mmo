use crossbeam_channel::{unbounded, Receiver};
use legion::{system, systems::CommandBuffer, Schedule};
use macroquad::{prelude::DARKBLUE, window::clear_background};

use crate::{
    ui::{
        add_ui_layout_systems, add_ui_rendering_systems,
        spawner::{spawn_button, spawn_dynamic_text, spawn_spacer, spawn_ui_container},
        UIConstraint, UIRoot, UISize,
    },
    ClearColor, Schedules,
};

#[derive(Copy, Clone)]
enum MainMenuEvent {
    PlayButtonClicked,
    QuitButtonClicked,
}

struct MainMenuEventHandler(Receiver<MainMenuEvent>);

pub fn main_menu_schedules() -> Schedules {
    let enter_schedule = Schedule::builder()
        .add_system(set_bg_color_system())
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

#[system]
fn handle_main_menu_events(#[resource] handler: &mut MainMenuEventHandler) {
    let receiver = &handler.0;
    receiver.try_iter().for_each(|event| match event {
        MainMenuEvent::PlayButtonClicked => println!("Play Button Clicked"),
        MainMenuEvent::QuitButtonClicked => println!("Quit Button Clicked"),
    });
}

#[system]
fn set_bg_color(#[resource] color: &mut ClearColor) {
    color.0 = DARKBLUE;
}

#[system]
fn spawn_main_menu(commands: &mut CommandBuffer) {
    println!("Spawning Main Menu Components...");

    let (sender, receiver) = unbounded();

    commands.exec_mut(move |_, resources| {
        let event_handler = MainMenuEventHandler(receiver.clone());
        resources.insert(event_handler);
    });

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
        sender.clone(),
        MainMenuEvent::PlayButtonClicked,
    );
    let quit_button = spawn_button(
        commands,
        "Quit",
        sender.clone(),
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

fn render_schedule() -> Schedule {
    let mut builder = Schedule::builder();
    builder.add_thread_local(draw_main_menu_system());

    add_ui_rendering_systems::<MainMenuEvent>(&mut builder);

    builder.build()
}

#[system]
fn draw_main_menu(#[resource] clear_color: &ClearColor) {
    clear_background(clear_color.0);
}
