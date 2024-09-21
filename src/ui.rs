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
use std::net::SocketAddr;
use askama::Template;
use axum::{
    Json,
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use log::info;
use serde::Deserialize;

use tower_http::services::ServeDir;

use crate::config;
use crate::SharedState;

pub async fn serve(port: u16, shared_state: SharedState) {
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/settings", get(show_settings))
        .route("/api/settings", post(post_settings))
        .route("/api/set_rc", post(post_rc))
        .nest_service("/static", ServeDir::new("static"))
        /* For an example for timeouts and tracing, have a look at the git history */
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    axum::serve(listener,
        app.into_make_service_with_connect_info::<SocketAddr>())
        .await.unwrap()
}

#[derive(PartialEq)]
enum ActivePage {
    Dashboard,
    Settings,
}

impl ActivePage {
    // Used by templates/head.html to include the correct js files in <head>
    fn styles(&self) -> Vec<&'static str> {
        match self {
            ActivePage::Dashboard => vec!["dashboard.js", "main.js"],
            ActivePage::Settings => vec!["settings.js", "main.js"],
        }
    }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate<'a> {
    title: &'a str,
    page: ActivePage,
    conf: config::Config,
    params: Vec<crate::dabmux::Param>,
    params_errors: Option<String>,
    stats: Option<crate::dabmux::Stats>,
    stats_errors: Option<String>,
}

async fn dashboard(State(state): State<SharedState>) -> DashboardTemplate<'static> {
    let (conf, params_result, stats_result) = {
        let mut st = state.lock().unwrap();

        let params_result = st.dabmux.get_rc_parameters();
        let stats_result = st.dabmux.get_stats();
        info!("STATS: {:?}", stats_result);

        (st.conf.clone(), params_result, stats_result)
    };

    let (params, params_errors) = match params_result {
        Ok(v) => (v, None),
        Err(e) => (Vec::new(), Some(format!("{}", e))),
    };

    let (stats, stats_errors) = match stats_result {
        Ok(v) => (Some(v), None),
        Err(e) => (None, Some(format!("{}", e))),
    };

    DashboardTemplate {
        title: "Dashboard",
        conf,
        page: ActivePage::Dashboard,
        params,
        params_errors,
        stats,
        stats_errors,
    }
}

#[derive(Deserialize)]
struct SetRc {
    pub module : String,
    pub param : String,
    pub value : String,
}

async fn post_rc(
    State(state): State<SharedState>,
    Json(set_rc): Json<SetRc>) -> (StatusCode, String) {

    let set_rc_result = {
        let mut st = state.lock().unwrap();
        st.dabmux.set_rc_parameter(&set_rc.module, &set_rc.param, &set_rc.value)
    };

    match set_rc_result {
        Ok(()) => (StatusCode::OK, "".to_owned()),
        Err(e) => (StatusCode::BAD_REQUEST, e.to_string()),
    }
}

#[derive(Template)]
#[template(path = "settings.html")]
struct SettingsTemplate<'a> {
    title: &'a str,
    page: ActivePage,
    conf: config::Config,
}

async fn show_settings(State(state): State<SharedState>) -> SettingsTemplate<'static> {
    SettingsTemplate {
        title: "Settings",
        page: ActivePage::Settings,
        conf: state.lock().unwrap().conf.clone(),
    }
}

async fn post_settings(
    State(state): State<SharedState>,
    Json(conf): Json<config::Config>) -> (StatusCode, String) {

    match conf.store() {
        Ok(()) => {
            state.lock().unwrap().conf.clone_from(&conf);

            match conf.write_dabmux_json() {
                Ok(()) => (StatusCode::OK, "".to_owned()),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to write odr-dabmux config: {}", e.to_string()))
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to write UI config: {}", e.to_string()))
    }
}
