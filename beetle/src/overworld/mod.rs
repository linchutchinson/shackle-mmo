mod player;
mod spawner;

use common::math::Vec2;
use legion::{system, systems::CommandBuffer, Schedule};
use macroquad::prelude::DARKBROWN;

use crate::{
    draw_clear_color_system,
    ui::{add_ui_layout_systems, add_ui_rendering_systems},
    ClearColor, Schedules,
};

use self::{
    player::{draw_player_system, move_player_system},
    spawner::{spawn_overworld_entities_system, spawn_overworld_ui_system},
};

const TILE_SIZE: f32 = 64.0;

pub fn overworld_schedules() -> Schedules {
    let enter_schedule = Schedule::builder()
        .add_system(initialize_overworld_resources_system())
        .flush()
        .add_system(spawn_overworld_ui_system())
        .add_system(spawn_overworld_entities_system())
        .build();

    let mut tick_sbuilder = Schedule::builder();
    tick_sbuilder.add_system(move_player_system());
    add_ui_layout_systems::<()>(&mut tick_sbuilder);
    let tick_schedule = tick_sbuilder.build();

    let mut render_sbuilder = Schedule::builder();
    render_sbuilder
        .add_thread_local(draw_clear_color_system())
        .add_thread_local(draw_player_system());
    add_ui_rendering_systems::<()>(&mut render_sbuilder);
    let render_schedule = render_sbuilder.build();

    Schedules {
        enter_schedule,
        tick_schedule,
        render_schedule,
    }
}

#[system]
fn initialize_overworld_resources(commands: &mut CommandBuffer) {
    commands.exec_mut(|_, resources| {
        let clear_color = ClearColor(DARKBROWN);
        resources.insert(clear_color);
    });
}

pub struct Position(Vec2);
