mod button;
mod container;
mod text;

pub mod spawner;

pub use container::{UIConstraint, UIRoot, UISize};
pub use text::Text;

use common::math::Rect;
use legion::{system, systems::Builder};
use macroquad::{prelude::RED, shapes::draw_rectangle_lines};

use self::{
    button::{draw_button_system, handle_button_input_system},
    container::{layout_ui_system, size_ui_root_system},
    text::{
        calculate_dynamic_font_size_system, handle_text_input_input_system,
        render_text_input_system, render_text_system,
    },
};

// TODO UI Schedules should use a stack resource to handle UI ordering
// as well as input handling.

pub fn add_ui_layout_systems<T: Send + Sync + Copy + 'static>(builder: &mut Builder) {
    builder
        .add_system(size_ui_root_system())
        .flush()
        .add_system(layout_ui_system())
        .flush()
        .add_system(handle_text_input_input_system(0))
        .add_system(handle_button_input_system::<T>())
        .add_system(calculate_dynamic_font_size_system())
        .flush();
}

pub fn add_ui_rendering_systems<T: Send + Sync + Copy + 'static>(builder: &mut Builder) {
    builder
        .flush()
        .add_thread_local(render_rect_outlines_system())
        .add_thread_local(render_text_input_system())
        .add_thread_local(draw_button_system::<T>())
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
