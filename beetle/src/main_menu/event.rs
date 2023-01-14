use crossbeam_channel::{unbounded, Receiver, Sender};
use legion::Entity;

#[derive(Copy, Clone)]
pub enum MainMenuEvent {
    ButtonClicked(MainMenuButton),
}

#[derive(Copy, Clone)]
pub enum MainMenuButton {
    Play,
    Quit,
    Login(Entity),
}

#[derive(Clone)]
pub enum MainMenuNotification {
    Error(String),
}

pub struct MainMenuEventHandler {
    event_channel: (Sender<MainMenuEvent>, Receiver<MainMenuEvent>),
    notification_channel: (Sender<MainMenuNotification>, Receiver<MainMenuNotification>),
}

impl MainMenuEventHandler {
    pub fn new() -> Self {
        let event_channel = unbounded();
        let notification_channel = unbounded();

        Self {
            event_channel,
            notification_channel,
        }
    }

    pub fn event_sender(&self) -> Sender<MainMenuEvent> {
        self.event_channel.0.clone()
    }

    pub fn event_receiver(&self) -> &Receiver<MainMenuEvent> {
        &self.event_channel.1
    }

    pub fn send_notification(&self, notification: MainMenuNotification) {
        self.notification_channel
            .0
            .send(notification)
            .expect("This should send.");
    }

    pub fn notification_receiver(&self) -> Receiver<MainMenuNotification> {
        self.notification_channel.1.clone()
    }
}

pub struct NotificationDisplay(pub Receiver<MainMenuNotification>);
