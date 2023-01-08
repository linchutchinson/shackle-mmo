use std::time::Instant;

use beetle::Application;
use macroquad::window::next_frame;

const TICKS_PER_SECOND: usize = 60;
const SECS_PER_TICK: f32 = 1.0 / TICKS_PER_SECOND as f32;

#[macroquad::main("Shackle MMO - Beetle V0.1")]
async fn main() {
    env_logger::init();
    let mut app = Application::new();

    // We start above 0.0 for elapsed just to run one step of game state before the
    // first render call.
    let mut elapsed = SECS_PER_TICK;
    while app.is_running {
        let frame_start = Instant::now();

        app.handle_input();

        while elapsed >= SECS_PER_TICK {
            app.tick();
            elapsed -= SECS_PER_TICK;
        }

        app.render();
        next_frame().await;

        elapsed += frame_start.elapsed().as_secs_f32();
    }
}
