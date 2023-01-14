use client::NetworkClient;
use common::{
    math::{Rect, Vec2},
    NetworkID, PLAY_AREA_SIZE,
};
use legion::{system, systems::CommandBuffer};
use macroquad::{
    prelude::{is_key_down, is_mouse_button_pressed, mouse_position, Color, BLACK, WHITE},
    text::{draw_text, measure_text},
    window::{screen_height, screen_width},
};

use crate::ui::spawner::{spawn_button, spawn_context_menu};

use super::{OverworldUIEvent, OverworldUIEventChannel, Position};

pub struct Player;
pub struct Controller;
pub struct WorldDisplay(pub String, pub Color);
pub struct NeedsName;
pub struct HoverName {
    pub radius: f32,
    pub name: String,
}

#[system(for_each)]
pub fn draw_world_objects(display: &WorldDisplay, pos: &Position) {
    let screen_width = screen_width();
    let screen_height = screen_height();
    let tl = Vec2::new(screen_width * 0.5, screen_height * 0.5) - PLAY_AREA_SIZE * 0.5;
    draw_text(
        &display.0,
        tl.x + pos.0.x - 16.0,
        tl.y + pos.0.y + 16.0,
        64.0,
        display.1,
    );
}

#[system(for_each)]
pub fn move_player(
    #[resource] client: &mut NetworkClient,
    _: &Player,
    _: &Controller,
    pos: &mut Position,
) {
    let x_dir = match (
        is_key_down(macroquad::prelude::KeyCode::A),
        is_key_down(macroquad::prelude::KeyCode::D),
    ) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    let y_dir = match (
        is_key_down(macroquad::prelude::KeyCode::W),
        is_key_down(macroquad::prelude::KeyCode::S),
    ) {
        (true, false) => -1.0,
        (false, true) => 1.0,
        _ => 0.0,
    };

    const PLAYER_SPEED: f32 = 4.0;
    let move_vec = Vec2::new(x_dir, y_dir) * PLAYER_SPEED;

    if move_vec != Vec2::ZERO {
        pos.0 += move_vec;

        // TODO: Handle network errors.
        let result = client.move_player(pos.0);

        if result.is_err() {
            log::error!("Error sending move packet. {:?}", result.unwrap_err());
        }
    }
}

pub fn world_to_screen(pos: Vec2) -> Vec2 {
    pos + Vec2::from((screen_width(), screen_height())) * 0.5 - PLAY_AREA_SIZE * 0.5
}

#[system(for_each)]
pub fn draw_hover_name(pos: &Position, hover_name: &HoverName) {
    let mouse_pos: Vec2 = mouse_position().into();

    let screen_pos = world_to_screen(pos.0);

    if screen_pos.distance_to(mouse_pos) <= hover_name.radius {
        let text_size = measure_text(&hover_name.name, None, 24, 1.0);
        draw_text(
            &hover_name.name,
            screen_pos.x - text_size.width * 0.5,
            screen_pos.y - hover_name.radius,
            24.0,
            WHITE,
        );
    }
}

pub struct OtherPlayer;

#[system(for_each)]
pub fn spawn_context_menu_when_rclicked(
    network_id: &NetworkID,
    pos: &Position,
    _: &OtherPlayer,
    #[resource] event_stream: &OverworldUIEventChannel,
    commands: &mut CommandBuffer,
) {
    const CLICK_RADIUS: f32 = 32.0;
    let screen_pos = world_to_screen(pos.0);
    let mouse_pos = mouse_position().into();

    if screen_pos.distance_to(mouse_pos) <= CLICK_RADIUS
        && is_mouse_button_pressed(macroquad::prelude::MouseButton::Right)
    {
        let duel_button = spawn_button(
            commands,
            "DUEL",
            event_stream.0.clone(),
            OverworldUIEvent::Challenge(*network_id),
        );
        let menu = spawn_context_menu(commands, &[duel_button]);
        commands.add_component(menu, Rect::new(mouse_pos.x, mouse_pos.y, 100.0, 200.0));
    }
}
