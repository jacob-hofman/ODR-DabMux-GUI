use std::sync::{Arc, Mutex};
use anyhow::{anyhow, Context};
use log::{debug, info, warn, error};

mod ui;
mod config;

struct AppState {
    conf : config::Config,
}

type SharedState = Arc<Mutex<AppState>>;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .env()
        .init().unwrap();

    let conf = config::Config::load().expect("Could not load config");

    let shared_state = Arc::new(Mutex::new(AppState {
        conf : conf.clone(),
    }));

    let port = 3000;
    info!("Setting up listener on port {port}");
    ui::serve(port, shared_state).await;
    Ok(())
}
