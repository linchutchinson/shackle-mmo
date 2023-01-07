pub mod spawner;
mod text;

use common::math::Rect;
use legion::{system, systems::Builder};
use macroquad::{prelude::RED, shapes::draw_rectangle_lines};

use self::text::{calculate_dynamic_font_size_system, render_text_system};

pub fn add_ui_layout_systems(builder: &mut Builder) {
    builder
        .flush()
        .add_system(calculate_dynamic_font_size_system())
        .flush();
}

pub fn add_ui_rendering_systems(builder: &mut Builder) {
    builder
        .flush()
        .add_thread_local(render_rect_outlines_system())
        .add_thread_local(render_text_system())
        .flush();
}

#[system(for_each)]
fn render_rect_outlines(rect: &Rect) {
    draw_rectangle_lines(
        rect.position.x,
        rect.position.y,
        rect.size.x,
        rect.size.y,
        2.0,
        RED,
    );
}
