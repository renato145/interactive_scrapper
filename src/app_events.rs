use crate::{
    gadget_selector::{get_user_selection, initialize_gadget_selector, UserSelection},
    HELP_MSG,
};
use anyhow::Result;
use fantoccini::{Client, Locator};
use std::sync::mpsc::Receiver;

#[derive(Debug)]
pub enum AppEvent {
    InitializeGadget,
    GetHelp,
    GetSelected,
    Quit,
}

pub async fn handle_app_events(client: &mut Client, rx: Receiver<AppEvent>) -> Result<()> {
    while let Ok(msg) = rx.recv() {
        match msg {
            AppEvent::InitializeGadget => {
                initialize_gadget_selector(client).await?;
            }
            AppEvent::GetSelected => {
                match get_user_selection(client).await? {
                    UserSelection::Empty => {
                        tracing::info!("No selector found");
                    }
                    UserSelection::Selection(selector) => {
                        tracing::info!("Got user selector: {:?}", selector);
                        let elements = client.find_all(Locator::Css(&selector)).await?;
                        tracing::info!("Found {} elements", elements.len());
                        // TODO: scrap items
                    }
                    UserSelection::MissingGadget => {
                        tracing::info!("No gadget present, initializing it...");
                        initialize_gadget_selector(client).await?;
                    }
                }
            }
            AppEvent::GetHelp => {
                println!("{}", HELP_MSG);
            }
            AppEvent::Quit => break,
        }
    }
    Ok(())
}
