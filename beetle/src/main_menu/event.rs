use crossbeam_channel::{Receiver, Sender};
use legion::Entity;

#[derive(Copy, Clone)]
pub enum MainMenuEvent {
    PlayButtonClicked,
    QuitButtonClicked,
    LoginButtonClicked(Entity),
}

pub struct MainMenuEventHandler(pub Receiver<MainMenuEvent>, pub Sender<MainMenuEvent>);
