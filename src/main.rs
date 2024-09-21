/*
 * A Configuration and Control UI for ODR-DabMux
 * Copyright (C) 2024 Matthias P. Braendli
 *
 * This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public
 * License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later
 * version.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied
 * warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

use std::sync::{Arc, Mutex};
use log::info;

mod ui;
mod config;
mod dabmux;

struct AppState {
    conf : config::Config,
    dabmux : dabmux::DabMux,
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
        dabmux : dabmux::DabMux::new(),
    }));

    let port = 3000;
    info!("Setting up listener on port {port}");
    ui::serve(port, shared_state).await;
    Ok(())
}
