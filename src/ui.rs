use std::net::SocketAddr;
use anyhow::{anyhow, Context};
use askama::Template;
use axum::{
    Form,
    Json,
    Router,
    extract::State,
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, ConnectInfo},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;

use log::{debug, info, warn, error};
use tower_http::services::ServeDir;

use crate::config;
use crate::SharedState;

pub async fn serve(port: u16, shared_state: SharedState) {
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/settings", get(show_settings).post(post_settings))
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
    None,
}

impl ActivePage {
    // Used by templates/head.html to include the correct js files in <head>
    fn styles(&self) -> Vec<&'static str> {
        match self {
            ActivePage::Dashboard => vec![],
            ActivePage::Settings => vec![],
            ActivePage::None => vec![],
        }
    }
}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate<'a> {
    title: &'a str,
    page: ActivePage,
    conf: config::Config,
}

async fn dashboard(State(state): State<SharedState>) -> DashboardTemplate<'static> {
    let conf = {
        let st = state.lock().unwrap();
        st.conf.clone()
    };

    DashboardTemplate {
        title: "Dashboard",
        conf,
        page: ActivePage::Dashboard,
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

#[derive(Template)]
#[template(path = "settings_applied.html")]
struct SettingsAppliedTemplate<'a> {
    title: &'a str,
    page: ActivePage,
    conf: config::Config,
    ok: bool,
    error_message: &'a str,
    error_reason: String,
}

#[derive(Deserialize, Debug)]
struct FormConfig {
    name: String,
}

impl TryFrom<FormConfig> for config::Config {
    type Error = anyhow::Error;

    fn try_from(value: FormConfig) -> Result<Self, Self::Error> {
        Ok(config::Config {
            name: value.name,
        })
    }
}

async fn post_settings(
    State(state): State<SharedState>,
    Form(input): Form<FormConfig>) -> (StatusCode, SettingsAppliedTemplate<'static>) {
    match config::Config::try_from(input) {
        Ok(c) => {
            match c.store() {
                Ok(()) => {
                    state.lock().unwrap().conf.clone_from(&c);

                    (StatusCode::OK, SettingsAppliedTemplate {
                        title: "Settings",
                        conf: c,
                        page: ActivePage::None,
                        ok: true,
                        error_message: "",
                        error_reason: "".to_owned(),
                    })
                }
                Err(e) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, SettingsAppliedTemplate {
                        title: "Settings",
                        conf : c,
                        page: ActivePage::None,
                        ok: false,
                        error_message: "Failed to store config",
                        error_reason: e.to_string(),
                    })
                },
            }
        },
        Err(e) => {
            (StatusCode::BAD_REQUEST, SettingsAppliedTemplate {
                        title: "Settings",
                        conf: state.lock().unwrap().conf.clone(),
                        page: ActivePage::None,
                        ok: false,
                        error_message: "Error interpreting POST data",
                        error_reason: e.to_string(),
                    })
        },
    }
}
