use crossbeam_channel::{Receiver, Sender};

#[derive(Copy, Clone)]
pub enum MainMenuEvent {
    PlayButtonClicked,
    QuitButtonClicked,
}

pub struct MainMenuEventHandler(pub Receiver<MainMenuEvent>, pub Sender<MainMenuEvent>);
