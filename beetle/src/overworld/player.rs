use common::math::Vec2;
use legion::system;
use macroquad::{
    prelude::{is_key_down, WHITE},
    text::draw_text,
};

use super::Position;

pub struct Player;

#[system(for_each)]
pub fn draw_player(_: &Player, pos: &Position) {
    draw_text("@", pos.0.x, pos.0.y, 64.0, WHITE);
}

#[system(for_each)]
pub fn move_player(_: &Player, pos: &mut Position) {
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

    pos.0 += move_vec;
}
