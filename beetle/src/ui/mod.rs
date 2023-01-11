mod button;
mod container;
mod spinner;
mod text;

pub mod spawner;

pub use container::{UIConstraint, UIRoot, UISize};
pub use spinner::Spinner;
pub use text::SubmitOnEnter;
pub use text::Text;

use common::math::Rect;
use legion::{system, systems::Builder};
use macroquad::{prelude::Color, shapes::draw_rectangle};

use self::text::handle_text_input_submit_on_enter_system;
use self::{
    button::{draw_button_system, handle_button_input_system},
    container::{layout_ui_system, size_ui_root_system},
    spinner::{draw_spinner_system, rotate_spinner_system},
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
        .add_system(rotate_spinner_system())
        .add_system(handle_text_input_submit_on_enter_system())
        .flush();
}

pub fn add_ui_rendering_systems<T: Send + Sync + Copy + 'static>(builder: &mut Builder) {
    builder
        .flush()
        // FIXME: Inconsistency between render and draw
        //.add_thread_local(render_rect_outlines_system())
        .add_thread_local(render_rect_lightener_system())
        .add_thread_local(draw_spinner_system())
        .add_thread_local(render_text_input_system())
        .add_thread_local(draw_button_system::<T>())
        .add_thread_local(render_text_system())
        .flush();
}

/*
TODO: Make this a debug toggleable feature.
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
*/

struct UILayer;

#[system(for_each)]
fn render_rect_lightener(rect: &Rect, _: &UILayer) {
    const PADDING: f32 = 8.0;
    draw_rectangle(
        rect.position.x - PADDING,
        rect.position.y - PADDING,
        rect.size.x + (PADDING * 2.0),
        rect.size.y + (PADDING * 2.0),
        Color::new(0.0, 0.0, 0.0, 0.2),
    );
}
