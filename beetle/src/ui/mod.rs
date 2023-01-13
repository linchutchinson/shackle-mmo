mod button;
mod container;
mod spinner;
mod text;

pub mod spawner;

use container::UIContainer;
pub use container::{FullscreenRoot, UIConstraint, UIRoot, UISize};
use legion::systems::CommandBuffer;
use legion::world::SubWorld;
use legion::Entity;
use legion::EntityStore;
use legion::Query;
use legion::TryRead;
use macroquad::prelude::is_mouse_button_pressed;
use macroquad::prelude::mouse_position;
use macroquad::prelude::RED;
use macroquad::shapes::draw_rectangle_lines;
pub use spinner::Spinner;
pub use text::SubmitOnEnter;
pub use text::Text;

use common::math::Rect;
use legion::{system, systems::Builder};
use macroquad::{prelude::Color, shapes::draw_rectangle};

use self::container::size_fullscreen_root_system;
use self::text::handle_text_input_submit_on_enter_system;
use self::{
    button::{draw_button_system, handle_button_input_system},
    container::layout_ui_system,
    spinner::{draw_spinner_system, rotate_spinner_system},
    text::{
        calculate_dynamic_font_size_system, handle_text_input_input_system,
        render_text_input_system, render_text_system,
    },
};

// TODO UI Schedules should use a stack resource to handle UI ordering
// as well as input handling.

pub struct DeleteOnClickOff;

pub fn add_ui_layout_systems<T: Send + Sync + Copy + 'static>(builder: &mut Builder) {
    builder
        .add_system(size_fullscreen_root_system())
        .flush()
        .add_system(layout_ui_system())
        .flush()
        .add_system(handle_text_input_input_system(0))
        .add_system(handle_button_input_system::<T>())
        .add_system(calculate_dynamic_font_size_system())
        .add_system(rotate_spinner_system())
        .add_system(handle_text_input_submit_on_enter_system())
        .add_system(delete_on_click_off_system())
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

//TODO: Make this a debug toggleable feature.
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

#[system]
#[read_component(Rect)]
fn delete_on_click_off(
    world: &mut SubWorld,
    query: &mut Query<(Entity, &Rect, &DeleteOnClickOff, TryRead<UIContainer>)>,
    commands: &mut CommandBuffer,
) {
    if is_mouse_button_pressed(macroquad::prelude::MouseButton::Left)
        || is_mouse_button_pressed(macroquad::prelude::MouseButton::Right)
    {
        query.iter(world).for_each(|(e, rect, _, container)| {
            println!("Checking mouse pos.");
            let mouse_pos = mouse_position().into();
            if !rect.contains(mouse_pos) {
                println!("Deleting container.");
                if let Some(c) = container {
                    delete_container_children_recursive(world, c, commands);
                }
                commands.remove(*e)
            }
        });
    }
}

fn delete_container_children_recursive(
    world: &SubWorld,
    container: &UIContainer,
    commands: &mut CommandBuffer,
) {
    container.children.iter().for_each(|e| {
        if let Ok(r) = world.entry_ref(*e) {
            let has_container = r.get_component::<UIContainer>();
            if let Ok(child_container) = has_container {
                delete_container_children_recursive(world, child_container, commands);
            }
            commands.remove(*e);
        }
    });
}
