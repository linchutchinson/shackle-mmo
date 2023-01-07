mod main_menu;
mod ui;

use std::collections::HashMap;

use legion::{Resources, Schedule, World};
use macroquad::{
    prelude::{Color, RED},
    window::clear_background,
};
use main_menu::main_menu_schedules;

pub struct Application {
    pub is_running: bool,
    world: World,
    resources: Resources,
    current_state: AppState,
    states: HashMap<AppState, Schedules>,
}

impl Application {
    pub fn new() -> Self {
        let world = World::default();
        let mut resources = Resources::default();
        resources.insert(NextState(Some(AppState::MainMenu)));
        resources.insert(ClearColor(RED));

        let current_state = AppState::Startup;
        let mut states = HashMap::new();
        states.insert(AppState::MainMenu, main_menu_schedules());
        Self {
            is_running: true,
            world,
            resources,
            current_state,
            states,
        }
    }

    pub fn handle_input(&mut self) {}

    pub fn tick(&mut self) {
        let mut run_enter_step = false;
        let mut next_state = self.resources.get_mut::<NextState>().unwrap();

        if let Some(new_state) = next_state.0 {
            if new_state != self.current_state {
                self.current_state = new_state;
                next_state.0 = None;
                run_enter_step = true;
            }
        }

        drop(next_state);

        let schedules = self.states.get_mut(&self.current_state).expect(&format!(
            "Missing schedules for state {:?}",
            self.current_state,
        ));

        if run_enter_step {
            schedules
                .enter_schedule
                .execute(&mut self.world, &mut self.resources);
        }

        schedules
            .tick_schedule
            .execute(&mut self.world, &mut self.resources);
    }

    pub fn render(&mut self) {
        let schedules = self.states.get_mut(&self.current_state).expect(&format!(
            "Missing schedules for state {:?}",
            self.current_state,
        ));

        schedules
            .render_schedule
            .execute(&mut self.world, &mut self.resources);
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum AppState {
    Startup,
    MainMenu,
}

struct NextState(Option<AppState>);

pub struct Schedules {
    enter_schedule: Schedule,
    tick_schedule: Schedule,
    render_schedule: Schedule,
}

struct ClearColor(Color);
