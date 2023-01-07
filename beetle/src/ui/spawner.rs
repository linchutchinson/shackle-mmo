use common::math::Rect;
use legion::{systems::CommandBuffer, Entity};

use super::text::DynamicText;

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
