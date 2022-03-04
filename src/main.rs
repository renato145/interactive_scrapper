use anyhow::{Context, Result};
use fantoccini::ClientBuilder;
use interactive_scrapper::{
    app_events::handle_app_events, listener::start_listener_task, HELP_MSG,
};
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

#[tokio::main]
async fn main() -> Result<()> {
    init_subscriber("info");

    let mut client = match ClientBuilder::native()
        .connect("http://localhost:4444")
        .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to connect to web driver: {}", e);
            println!(
                "To run chromedriver use:\n> chromedriver --port=4444 --disable-dev-shm-usage\n\
                 If you need to install it:\n> sudo apt install chromium-browser chromium-chromedriver"
            );
            std::process::exit(2);
        }
    };
    println!("{}", HELP_MSG);

    let (tx, rx) = mpsc::channel();
    start_listener_task(tx);
    handle_app_events(&mut client, rx).await?;

    tracing::info!("Quitting the app...");
    client.close().await.context("Failed to close web driver")
}
