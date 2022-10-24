use std::net::SocketAddr;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::RawBody;
use axum::routing::post;
use axum::Router;
use serde::Serialize;
use tokio::fs;
use tokio::process::Command;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

use common::errors::{timeout_or_500, ApiError};
use common::init_tracing;
use common::response::Bson;
use lazy_static::lazy_static;

lazy_static! {
    static ref APP_DIR: String =
        std::env::var("APP_DIR").unwrap_or_else(|_| "../../app".to_string());
    static ref TRUNK_BIN: String =
        std::env::var("TRUNK_BIN").unwrap_or_else(|_| "trunk".to_string());
    static ref PORT: u16 = std::env::var("PORT")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(4000);
}

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

async fn run(RawBody(body): RawBody) -> Result<Bson<Response>, ApiError> {
    let body = hyper::body::to_bytes(body).await.unwrap();
    if body.is_empty() {
        return Err(ApiError::NoBody);
    }
    let body = String::from_utf8_lossy(&body);
    let app_dir = fs::canonicalize(&*APP_DIR).await?;
    fs::write(app_dir.join("src/main.rs"), &*body).await?;
    let mut cmd = Command::new(&*TRUNK_BIN);
    let cmd = cmd
        .arg("--config")
        .arg(app_dir.join("Trunk.toml"))
        .arg("build");
    debug!(?cmd, "running command");
    let output = cmd.output().await?;
    if !output.status.success() {
        return Ok(Bson(Response::CompileError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        )));
    }

    let dist = app_dir.join("dist");
    let index_html = fs::read_to_string(dist.join("index.html")).await?;
    let js = fs::read_to_string(dist.join("app.js")).await?;
    let wasm = fs::read(dist.join("app_bg.wasm")).await?;

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
                .timeout(Duration::from_secs(10)),
        )
        .layer(GlobalConcurrencyLimitLayer::new(1))
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), *PORT);
    info!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
