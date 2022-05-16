use std::borrow::Cow;
use std::net::SocketAddr;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{routing::post, BoxError, Json, Router};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::process::Command;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, trace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use response::Bson;
use crate::errors::ApiError;
use lazy_static::lazy_static;

mod response;
mod errors;


lazy_static! {
    static ref APP_DIR: String = std::env::var("APP_DIR").unwrap_or_else(|_| "../app".to_string());
    static ref TRUNK_BIN: String = std::env::var("TRUNK_BIN").unwrap_or_else(|_| "trunk".to_string());
    static ref PORT: u16 = std::env::var("PORT").ok().and_then(|it| it.parse().ok()).unwrap_or_else(|| 3000);
}

#[derive(Deserialize)]
struct RunPayload {
    main_contents: String,
}

#[derive(Serialize)]
struct RunResponse {
    index_html: String,
    js: String,
    wasm: Vec<u8>,
}


async fn run(Json(body): Json<RunPayload>) -> Result<Bson<RunResponse>, ApiError> {
    let app_dir = fs::canonicalize(&*APP_DIR).await?;

    fs::write(app_dir.join("src/main.rs"), &body.main_contents).await?;
    let mut cmd = Command::new(&*TRUNK_BIN);
    let cmd = cmd.arg("build")
        .arg(app_dir.join("index.html"))
        .arg("--release");
    debug!(?cmd, "running command");
    let output = cmd.output().await?;
    if !output.status.success() {
        return Err(ApiError::BuildFailed(output))
    }

    let mut dist = fs::read_dir(app_dir.join("dist")).await?;
    let mut index_html = None;
    let mut js = None;
    let mut wasm = None;
    while let Some(file) = dist.next_entry().await? {
        let filename = file.file_name();
        let filename = filename.to_string_lossy();
        if filename.ends_with(".html") {
            index_html = Some(fs::read_to_string(file.path()).await?);
        } else if filename.ends_with(".js") {
            js = Some(fs::read_to_string(file.path()).await?);
        } else if filename.ends_with(".wasm") {
            wasm = Some(fs::read(file.path()).await?);
        }
    }

    let body = RunResponse {
        index_html: index_html.ok_or_else(|| ApiError::BuildFileNotFound("index.html"))?,
        js: js.ok_or_else(|| ApiError::BuildFileNotFound("index.js"))?,
        wasm: wasm.ok_or_else(|| ApiError::BuildFileNotFound("index.wasm"))?,
    };
    Ok(Bson(body))
}

async fn hello() -> Bson<RunResponse> {
    Bson(RunResponse {
        index_html: "index_html".to_string(),
        js: "js".to_string(),
        wasm: "wasm".as_bytes().to_vec(),
    })
}

async fn handle_errors(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

async fn trunk_version() -> String {
    Command::new(&*TRUNK_BIN)
        .arg("--version")
        .output()
        .await
        .map(|v| String::from_utf8_lossy(&v.stdout).to_string())
        .unwrap_or_else(|_| "failed to get trunk version".to_string())
}

#[tokio::main]
async fn main() {
    let app_dir = &*APP_DIR;
    let trunk_path = &*TRUNK_BIN;

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "backend=trace,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    debug!(?app_dir);
    let trunk_version = trunk_version().await;
    debug!(trunk_bin_path = ?trunk_path, trunk_version = ?trunk_version);

    // build our application with a single route
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/run", post(run))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_errors))
                .timeout(Duration::from_secs(10)),
        )
        .layer(GlobalConcurrencyLimitLayer::new(1))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), *PORT);
    // run it with hyper on localhost:3000
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
