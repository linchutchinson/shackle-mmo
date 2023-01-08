use common::math::Rect;
use crossbeam_channel::Sender;
use legion::system;
use macroquad::{
    prelude::{
        is_mouse_button_down, is_mouse_button_pressed, mouse_position, DARKGRAY, GRAY, LIGHTGRAY,
    },
    shapes::draw_rectangle,
};

pub struct Button<T: Send + Sync + Copy + 'static> {
    state: ButtonState,
    sender: Sender<T>,
    event: T,
}

impl<T: Send + Sync + Copy> Button<T> {
    pub fn new(sender: Sender<T>, event: T) -> Self {
        Self {
            state: ButtonState::Normal,
            sender,
            event,
        }
    }
}

#[derive(PartialEq)]
enum ButtonState {
    Normal,
    Hover,
    Click,
}

#[system(for_each)]
pub fn handle_button_input<T: Send + Sync + Copy + 'static>(button: &mut Button<T>, rect: &Rect) {
    let mouse_pos = mouse_position();

    if rect.contains(mouse_pos.into()) {
        if button.state != ButtonState::Click
            && is_mouse_button_pressed(macroquad::prelude::MouseButton::Left)
        {
            button.state = ButtonState::Click;
            button
                .sender
                .send(button.event)
                .expect("A button sent a message to a channel that is no longer connected!");
        } else if !is_mouse_button_down(macroquad::prelude::MouseButton::Left) {
            button.state = ButtonState::Hover;
        }
    } else {
        if !(button.state == ButtonState::Click
            && is_mouse_button_down(macroquad::prelude::MouseButton::Left))
        {
            button.state = ButtonState::Normal
        }
    }
}

#[system(for_each)]
pub fn draw_button<T: Send + Sync + Copy + 'static>(button: &Button<T>, rect: &Rect) {
    let color = match button.state {
        ButtonState::Normal => GRAY,
        ButtonState::Hover => LIGHTGRAY,
        ButtonState::Click => DARKGRAY,
    };

    draw_rectangle(rect.left(), rect.top(), rect.size.x, rect.size.y, color);
}
