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

use response::Bson;
use crate::errors::ApiError;

mod response;
mod errors;

const APP_DIR: &str = match option_env!("APP_DIR") {
    None => "../app",
    Some(v) => v,
};

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
    let app_dir = fs::canonicalize(APP_DIR).await?;

    fs::write(app_dir.join("src/main.rs"), &body.main_contents).await?;
    let output = Command::new("/home/hamza/bin/trunk")
        .arg("build")
        .arg(app_dir.join("index.html"))
        .arg("--release")
        .output()
        .await?;
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

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/run", post(run))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_errors))
                .timeout(Duration::from_secs(10)),
        )
        .layer(GlobalConcurrencyLimitLayer::new(1));

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
