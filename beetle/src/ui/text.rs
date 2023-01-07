use common::math::Rect;
use legion::{system, systems::CommandBuffer, Entity};
use macroquad::{
    prelude::BLACK,
    text::{draw_text, measure_text},
};

pub struct DynamicText(pub String);
pub struct Text(String, f32);

#[system(for_each)]
pub fn calculate_dynamic_font_size(
    entity: &Entity,
    dynamic_text: &DynamicText,
    rect: &Rect,
    commands: &mut CommandBuffer,
) {
    const MIN_FONT_SIZE: u16 = 8;
    const MAX_FONT_SIZE: u16 = 256;
    fn calculate_font_size_iter(text: &str, rect: &Rect, min: u16, max: u16) -> u16 {
        if min >= max {
            return min;
        }
        let middle = (min + max + 1) / 2;

        let text_size = measure_text(text, None, middle, 1.0);
        let contained_in_rect = text_size.width <= rect.size.x && text_size.height <= rect.size.y;

        if contained_in_rect {
            // Try larger size.
            calculate_font_size_iter(text, rect, middle + 1, max)
        } else {
            // Try smaller size.
            calculate_font_size_iter(text, rect, min, middle - 1)
        }
    }

    let font_size =
        calculate_font_size_iter(&dynamic_text.0, rect, MIN_FONT_SIZE, MAX_FONT_SIZE) as f32;

    // FIXME This is a string copy every frame which I think may be unneccessarily heavy.
    // There's probably some way to do this with a shared reference or only adjusting
    // the font size. Maybe change detection on the rect too could work.
    commands.add_component(*entity, Text(dynamic_text.0.clone(), font_size));
}

#[system(for_each)]
pub fn render_text(text: &Text, rect: &Rect) {
    let text_size = measure_text(&text.0, None, text.1 as u16, 1.0);
    let center = rect.center();
    let x = center.x - (text_size.width * 0.5);
    let y = center.y + (text_size.height * 0.5);
    draw_text(&text.0, x, y, text.1, BLACK);
}
