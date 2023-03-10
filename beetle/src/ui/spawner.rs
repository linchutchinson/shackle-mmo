use common::math::Rect;
use crossbeam_channel::Sender;
use legion::{systems::CommandBuffer, Entity};

use super::{
    button::Button,
    container::UIContainer,
    text::{DynamicText, Text, TextInput},
    DeleteOnClick, UIConstraint, UILayer, UIRoot, UISize,
};

/// Spawn a text object that grows or shrinks its font size to fit within a rect.
/// Note that a Rect is added by default but you're expected to add your own,
/// either manually or by using the text object
/// as part of a Container.
pub fn spawn_dynamic_text(commands: &mut CommandBuffer, text: &str) -> Entity {
    commands.push((
        UISize::Grow(1),
        DynamicText,
        Text(text.to_string(), 0.0),
        Rect::new(100.0, 100.0, 100.0, 100.0),
    ))
}

pub fn spawn_ui_container(commands: &mut CommandBuffer, children: &[Entity]) -> Entity {
    let container = UIContainer::default().with_children(children);

    commands.push((container, UISize::Grow(1)))
}

pub fn spawn_ui_panel(commands: &mut CommandBuffer, children: &[Entity]) -> Entity {
    let container = spawn_ui_container(commands, children);

    commands.add_component(container, UILayer);

    container
}

pub fn spawn_spacer(commands: &mut CommandBuffer) -> Entity {
    commands.push((UISize::Grow(1),))
}

pub fn spawn_button<T: Send + Sync + Copy + 'static>(
    commands: &mut CommandBuffer,
    text: &str,
    sender: Sender<T>,
    message: T,
) -> Entity {
    let label = spawn_dynamic_text(commands, text);
    let container = UIContainer::default()
        .with_margin(32.0)
        .with_children(&[label]);

    commands.push((
        container,
        UISize::Grow(1),
        UIConstraint::width_constraint(256.0),
        Button::new(sender, message),
    ))
}

pub fn spawn_text_input(commands: &mut CommandBuffer) -> Entity {
    commands.push((TextInput::new(), Text::empty(32.0), UISize::Grow(1)))
}

pub fn spawn_context_menu(commands: &mut CommandBuffer, children: &[Entity]) -> Entity {
    let ctx_menu = spawn_ui_container(commands, children);

    commands.add_component(ctx_menu, UIRoot);
    commands.add_component(ctx_menu, UILayer);
    commands.add_component(ctx_menu, DeleteOnClick);

    ctx_menu
}
