use common::math::Rect;
use legion::{systems::CommandBuffer, Entity};

use super::{text::DynamicText, Layout, UIContainer, UIRoot};

/// Spawn a text object that grows or shrinks its font size to fit within a rect.
/// Note that a Rect is added by default but you're expected to add your own,
/// either manually or by using the text object
/// as part of a Container.
pub fn spawn_dynamic_text(commands: &mut CommandBuffer, text: &str) -> Entity {
    commands.push((
        DynamicText(text.to_string()),
        Rect::new(100.0, 100.0, 100.0, 100.0),
    ))
}

pub fn spawn_ui_container(
    commands: &mut CommandBuffer,
    layout: Layout,
    children: &[Entity],
) -> Entity {
    let children = children.into();
    let container = UIContainer {
        layout,
        children,
        margin: 4.0,
        gap: 4.0,
    };

    commands.push((container,))
}
