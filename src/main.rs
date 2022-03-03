use anyhow::{Context, Result};
use fantoccini::{ClientBuilder, Locator};
use rdev::{listen, Event, EventType, Key};
use std::sync::mpsc;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, EnvFilter, Registry};

fn init_subscriber(env_filter: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let fmt_layer = tracing_subscriber::fmt::layer();
    let subscriber = Registry::default().with(env_filter).with(fmt_layer);
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[derive(Debug)]
enum AppEvent {
    GetSelected,
    Quit,
}

const HELP_MSG: &str = r#"INSTRUCTIONS:
Use the selector gadget to set the elements to scrape, then...

HOTKEYS:
-  A : Get selected elements. 
- Esc: quit the program."#;

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
                Key::Escape => tx.send(AppEvent::Quit).unwrap(),
                Key::KeyA => tx.send(AppEvent::GetSelected).unwrap(),
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

    client
        .goto("https://en.wikipedia.org/wiki/Foobar")
        .await
        .context("Failed to navigate to url")?;
    let url = client.current_url().await?;
    tracing::info!("Setting selector gadget script...");
    client
        // https://selectorgadget.com/
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
    tracing::info!("Page {} ready", url);

    while let Ok(msg) = rx.recv() {
        match msg {
            AppEvent::GetSelected => {
                let user_selector = client
                    .find(Locator::Id("_sg_path_field"))
                    .await?
                    .prop("value")
                    .await?
                    .map(|s| {
                        if s == "No valid path found." {
                            None
                        } else {
                            Some(s)
                        }
                    })
                    .flatten();

                if let Some(selector) = user_selector {
                    tracing::info!("Got user selector: {:?}", selector);
                    let elements = client.find_all(Locator::Css(&selector)).await?;
                    tracing::info!("Found {} elements", elements.len());
                } else {
                    tracing::info!("No selector found");
                }
            }
            AppEvent::Quit => break,
        }
    }

    tracing::info!("Quitting the app...");
    client.close().await.context("Failed to close web driver")
}
