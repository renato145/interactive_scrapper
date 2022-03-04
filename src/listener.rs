use crate::app_events::AppEvent;
use rdev::{listen, Event, EventType, Key};
use std::{sync::mpsc::Sender, thread::JoinHandle};

struct AltKeyHandler {
    pressed: bool,
}

impl AltKeyHandler {
    fn new() -> Self {
        Self { pressed: false }
    }

    fn press(&mut self) {
        self.pressed = true;
    }

    fn release(&mut self) {
        self.pressed = false;
    }

    fn is_pressed(&self) -> bool {
        self.pressed
    }
}

fn get_listener_callback(tx: Sender<AppEvent>) -> impl FnMut(Event) {
    let mut alt_handler = AltKeyHandler::new();
    move |event: Event| {
        tracing::debug!("Received event: {:?}", event.event_type);
        match event.event_type {
            EventType::KeyPress(key) => {
                if let Key::Alt = key {
                    alt_handler.press();
                } else if alt_handler.is_pressed() {
                    match key {
                        Key::KeyS => tx.send(AppEvent::InitializeGadget).unwrap(),
                        Key::KeyA => tx.send(AppEvent::GetSelected).unwrap(),
                        Key::KeyH => tx.send(AppEvent::GetHelp).unwrap(),
                        Key::Escape => tx.send(AppEvent::Quit).unwrap(),
                        key => {
                            tracing::debug!("Unhandled KeyPress event: {:?}", key);
                        }
                    }
                    alt_handler.release();
                }
            }
            EventType::KeyRelease(Key::Alt) => alt_handler.release(),
            _ => (),
        }
    }
}

#[tracing::instrument(skip(tx))]
pub fn start_listener_task(tx: Sender<AppEvent>) -> JoinHandle<()> {
    let cb = get_listener_callback(tx);
    std::thread::spawn(|| {
        tracing::info!("Started");
        if let Err(error) = listen(cb) {
            tracing::error!("Error: {:?}", error)
        }
    })
}
