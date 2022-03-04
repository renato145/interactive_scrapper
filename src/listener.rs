use crate::app_events::AppEvent;
use rdev::{listen, Event, EventType, Key};
use std::{sync::mpsc::Sender, thread::JoinHandle};

pub fn get_listener_callback(tx: Sender<AppEvent>) -> impl Fn(Event) -> () {
    move |event: Event| {
        if let EventType::KeyPress(key) = event.event_type {
            match key {
                Key::KeyS => tx.send(AppEvent::InitializeGadget).unwrap(),
                Key::KeyA => tx.send(AppEvent::GetSelected).unwrap(),
                Key::KeyH => tx.send(AppEvent::GetHelp).unwrap(),
                Key::Escape => tx.send(AppEvent::Quit).unwrap(),
                key => {
                    tracing::debug!("Unhandled KeyPress event: {:?}", key);
                }
            }
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
