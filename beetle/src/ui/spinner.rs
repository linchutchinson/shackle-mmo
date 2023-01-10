use common::math::Rect;
use legion::system;
use macroquad::{
    prelude::{BLACK, ORANGE},
    shapes::{draw_circle, draw_line},
};

use crate::TICKS_PER_SECOND;

pub struct Spinner(f32);

const ROTATIONS_PER_SECOND: f32 = 0.4;
const ROTATIONS_PER_TICK: f32 = ROTATIONS_PER_SECOND / TICKS_PER_SECOND as f32;
const RADIANS_PER_TICK: f32 = 2.0 * std::f32::consts::PI * ROTATIONS_PER_TICK;

impl Spinner {
    pub fn new() -> Self {
        Self(0.0)
    }
}

#[system(for_each)]
pub fn rotate_spinner(spinner: &mut Spinner) {
    spinner.0 = (spinner.0 + RADIANS_PER_TICK) % (2.0 * std::f32::consts::PI);
}

const SPINNER_SEGMENTS: usize = 6;
const FIRST_SEGMENT_SIZE_PCT: f32 = 0.2;
const SMALLER_SEGMENT_SIZE_MULTIPLIER: f32 = 0.75;

const SEGMENT_ANGLE_OFFSET: f32 = std::f32::consts::PI * 2.0 * 0.1;

#[system(for_each)]
pub fn draw_spinner(spinner: &Spinner, rect: &Rect) {
    let center = rect.center();

    // The radius is the smaller axis of the container rect.
    let radius = rect.size.x.min(rect.size.y);

    let initial_segment_radius = radius * FIRST_SEGMENT_SIZE_PCT;
    for i in 0..SPINNER_SEGMENTS {
        let angle = spinner.0 - SEGMENT_ANGLE_OFFSET * i as f32;
        let segment_radius =
            initial_segment_radius * SMALLER_SEGMENT_SIZE_MULTIPLIER.powi(i as i32);
        let end_x = center.x + angle.cos() * (radius - initial_segment_radius);
        let end_y = center.y + angle.sin() * (radius - initial_segment_radius);

        draw_circle(end_x, end_y, segment_radius, BLACK);
    }
}
