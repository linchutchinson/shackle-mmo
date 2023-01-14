use common::math::Rect;
use crossbeam_channel::Sender;
use legion::system;
use macroquad::{
    prelude::{
        get_char_pressed, is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position,
        BLACK, DARKGRAY, GRAY,
    },
    shapes::draw_rectangle,
    text::{draw_text, measure_text},
};

pub struct DynamicText;
pub struct Text(pub String, pub f32);

impl Text {
    pub fn empty(font_size: f32) -> Self {
        Self("".to_string(), font_size)
    }
}

pub trait TextReceiver {
    fn send(text: &str);
}

pub struct SubmitOnEnter(pub Sender<String>);
pub struct TextInput {
    state: TextInputState,
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            state: TextInputState::Normal,
        }
    }
}

enum TextInputState {
    Normal,
    Focus,
}

#[system(for_each)]
pub fn calculate_dynamic_font_size(_: &DynamicText, text: &mut Text, rect: &Rect) {
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

    let font_size = calculate_font_size_iter(&text.0, rect, MIN_FONT_SIZE, MAX_FONT_SIZE) as f32;

    text.1 = font_size;
}

#[system(for_each)]
pub fn handle_text_input_input(
    input: &mut TextInput,
    text: &mut Text,
    rect: &Rect,
    #[state] backspace_delay: &mut usize,
) {
    match input.state {
        TextInputState::Normal => {
            // Check if focus acquired
            if rect.contains(mouse_position().into())
                && is_mouse_button_pressed(macroquad::prelude::MouseButton::Left)
            {
                input.state = TextInputState::Focus;
                // Clear existing char inputs
                // FIXME This seems suspicious.
                while get_char_pressed().is_some() {}
            }
        }
        TextInputState::Focus => {
            // Take Text Input
            while let Some(c) = get_char_pressed() {
                if c.is_ascii() {
                    text.0.push(c);
                }
            }

            if is_key_pressed(macroquad::prelude::KeyCode::Backspace) {
                text.0.pop();
            }

            const REPEAT_DELAY: usize = 10;
            if is_key_down(macroquad::prelude::KeyCode::Backspace) {
                if *backspace_delay < REPEAT_DELAY {
                    *backspace_delay += 1;
                } else {
                    text.0.pop();
                }
            } else {
                *backspace_delay = 0;
            }

            // Check if focus lost
            if !rect.contains(mouse_position().into())
                && is_mouse_button_pressed(macroquad::prelude::MouseButton::Left)
            {
                input.state = TextInputState::Normal;
            }
        }
    }
}

#[system(for_each)]
pub fn handle_text_input_submit_on_enter(
    input: &TextInput,
    submitter: &SubmitOnEnter,
    text: &mut Text,
) {
    if let TextInputState::Focus = input.state {
        if is_key_pressed(macroquad::prelude::KeyCode::Enter) {
            let result = submitter.0.send(text.0.clone());
            text.0.clear();

            if result.is_err() {
                log::error!(
                    "There was an error submitting text from the selected input. {}",
                    result.unwrap_err()
                );
            }
        }
    }
}

#[system(for_each)]
pub fn render_text(text: &Text, rect: &Rect) {
    let text_size = measure_text(&text.0, None, text.1 as u16, 1.0);
    let center = rect.center();
    let x = center.x - (text_size.width * 0.5);
    let y = center.y + (text_size.height * 0.5);
    draw_text(&text.0, x, y, text.1, BLACK);
}

#[system(for_each)]
pub fn render_text_input(input: &TextInput, rect: &Rect) {
    let color = match input.state {
        TextInputState::Normal => DARKGRAY,
        TextInputState::Focus => GRAY,
    };

    // FIXME Rewriting Draw statements for rectangles over and over like this is error prone. Should be a trait.
    draw_rectangle(rect.left(), rect.top(), rect.size.x, rect.size.y, color);
}
