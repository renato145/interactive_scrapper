use anyhow::{Context, Result};
use fantoccini::{Client, ClientBuilder, Locator};
use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

const HELP_MSG: &str = r#"INSTRUCTIONS:
1. Navigate to a web page and initialize the selector gadget by pressing `S`.
2. Use the selector gadget to set the elements to scrape.
3. Press `A` to get the selected elements.

HOTKEYS:
-  S : Initialize selector gadget.
-  A : Get selected elements. 
-  H : Show this help message. 
- Esc: quit the program."#;

fn init_subscriber(env_filter: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let fmt_layer = tracing_subscriber::fmt::layer();
    let subscriber = Registry::default().with(env_filter).with(fmt_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[derive(Debug)]
enum AppEvent {
    InitializeGadget,
    GetHelp,
    GetSelected,
    Quit,
}

#[derive(Debug)]
enum UserSelection {
    Empty,
    Selection(String),
    MissingGadget,
}

async fn get_user_selection(
    client: &mut Client,
) -> Result<UserSelection, fantoccini::error::CmdError> {
    match client
        .find_all(Locator::Id("_sg_path_field"))
        .await?
        .get_mut(0)
    {
        Some(gadget) => {
            match gadget
                .prop("value")
                .await?
                .map(|s| {
                    if s == "No valid path found." {
                        None
                    } else {
                        Some(s)
                    }
                })
                .flatten()
            {
                Some(selection) => Ok(UserSelection::Selection(selection)),
                None => Ok(UserSelection::Empty),
            }
        }
        None => Ok(UserSelection::MissingGadget),
    }
}

/// Initialize https://selectorgadget.com on current page
#[tracing::instrument(skip(client))]
async fn initialize_gadget_selector(
    client: &mut Client,
) -> Result<(), fantoccini::error::CmdError> {
    if let UserSelection::MissingGadget = get_user_selection(client).await? {
        tracing::info!("Setting selector gadget script...");
        client
            .execute_async(
                r#"
                const [callback] = arguments;
                (function () {
                var s = document.createElement("div");
                s.innerHTML = "Loading...";
                s.style.color = "black";
                s.style.padding = "20px";
                s.style.position = "fixed";
                s.style.zIndex = "9999";
                s.style.fontSize = "3.0em";
                s.style.border = "2px%20solid%20black";
                s.style.right = "40px";
                s.style.top = "40px";
                s.setAttribute("class", "selector_gadget_loading");
                s.style.background = "white";
                document.body.appendChild(s);
                s = document.createElement("script");
                s.setAttribute("type", "text/javascript");
                s.setAttribute(
                    "src",
                    "https://dv0akt2986vzh.cloudfront.net/unstable/lib/selectorgadget.js"
                );
                document.body.appendChild(s);
                })();
                callback();
                "#,
                vec![],
            )
            .await?;
        tracing::info!("Selector gadget ready");
    } else {
        tracing::info!("Gadget already initialized...");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_subscriber("info");

    let mut client = ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
        .expect("Failed to connect to web driver");

    println!("{HELP_MSG}");

    let (tx, rx) = mpsc::channel();
    let listener_callback = move |event: Event| {
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
    };
    let _listener_task = std::thread::spawn(|| {
        tracing::info!("Started listener task");
        if let Err(error) = listen(listener_callback) {
            tracing::error!("Error: {:?}", error)
        }
    });

    while let Ok(msg) = rx.recv() {
        match msg {
            AppEvent::InitializeGadget => {
                initialize_gadget_selector(&mut client).await?;
            }
            AppEvent::GetSelected => {
                match get_user_selection(&mut client).await? {
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
                        initialize_gadget_selector(&mut client).await?;
                    }
                }
            }
            AppEvent::GetHelp => {
                println!("{HELP_MSG}");
            }
            AppEvent::Quit => break,
        }
    }

    tracing::info!("Quitting the app...");
    client.close().await.context("Failed to close web driver")
}
