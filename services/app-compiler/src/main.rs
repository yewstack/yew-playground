use std::sync::LazyLock;
use std::time::Duration;

use axum::body::Bytes;
use axum::error_handling::HandleErrorLayer;
use axum::routing::post;
use axum::Router;
use serde::Serialize;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::process::Command;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

use common::errors::{timeout_or_500, ApiError};
use common::init_tracing;
use common::response::Bson;

static APP_DIR: LazyLock<String> =
    LazyLock::new(|| std::env::var("APP_DIR").unwrap_or_else(|_| "../../app".to_string()));
static TRUNK_BIN: LazyLock<String> =
    LazyLock::new(|| std::env::var("TRUNK_BIN").unwrap_or_else(|_| "trunk".to_string()));
static PORT: LazyLock<u16> = LazyLock::new(|| {
    std::env::var("PORT")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(4000)
});

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Output {
        index_html: String,
        js: String,
        wasm: Vec<u8>,
    },
    CompileError(String),
}

async fn run(body: Bytes) -> Result<Bson<Response>, ApiError> {
    if body.is_empty() {
        return Err(ApiError::NoBody);
    }
    let body = String::from_utf8_lossy(&body);
    let app_dir = match fs::canonicalize(&*APP_DIR).await {
        Ok(v) => v,
        Err(e) => {
            error!(?e, "failed to canonicalize app_dir path");
            return Err(ApiError::IoError(e))
        },
    };

    match fs::write(app_dir.join("src/main.rs"), &*body).await {
        Ok(_) => {},
        Err(e) => {
            error!(?e, "failed to write main.rs");
            return Err(ApiError::IoError(e))
        },
    };

    let mut cmd = Command::new(&*TRUNK_BIN);
    let cmd = cmd
        .arg("--config")
        .arg(app_dir.join("Trunk.toml"))
        .arg("build")
        .kill_on_drop(true);
    debug!(?cmd, "running command");

    let output = match cmd.output().await {
        Ok(o) => o,
        Err(e) => {
            error!(?e, "running trunk failed");
            return Err(ApiError::IoError(e))
        },
    };

    if !output.status.success() {
        return Ok(Bson(Response::CompileError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )));
    }

    let dist = app_dir.join("dist");
    let index_html = fs::read_to_string(dist.join("index.html")).await.map_err(|e| {
        error!(?e, "failed to read index.html");
        ApiError::IoError(e)
    })?;
    let js = fs::read_to_string(dist.join("app.js")).await.map_err(|e| {
        error!(?e, "failed to read app.js");
        ApiError::IoError(e)
    })?;
    let wasm = fs::read(dist.join("app_bg.wasm")).await.map_err(|e| {
        error!(?e, "failed to read app_bg.wasm");
        ApiError::IoError(e)
    })?;
    

    Ok(Bson(Response::Output {
        index_html,
        js,
        wasm,
    }))
}

async fn trunk_version() -> String {
    Command::new(&*TRUNK_BIN)
        .arg("--version")
        .output()
        .await
        .map(|v| String::from_utf8_lossy(&v.stdout).trim().to_string())
        .unwrap_or_else(|_| "failed to get trunk version".to_string())
}

#[tokio::main]
async fn main() {
    let app_dir = &*APP_DIR;
    let trunk_path = &*TRUNK_BIN;

    init_tracing();

    debug!(?app_dir);
    let trunk_version = trunk_version().await;
    debug!(trunk_bin_path = ?trunk_path, trunk_version = ?trunk_version);

    // build our application with a single route
    let app = Router::new()
        .route("/run", post(run))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(timeout_or_500))
                .timeout(Duration::from_secs(60)),
        )
        .layer(GlobalConcurrencyLimitLayer::new(1))
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", *PORT);
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
