use std::time::Instant;

use beetle::{Application, SECS_PER_TICK};
use macroquad::window::{next_frame, Conf};

fn window_conf() -> Conf {
    Conf {
        window_title: "Shackle MMO - Beetle V0.1".to_string(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
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
