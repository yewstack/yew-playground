use std::sync::LazyLock;
use std::time::Duration;

use anyhow::anyhow;
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::extract::Query;
use axum::response::Html;
use axum::routing::get;
use serde::Deserialize;
use tokio::fs;
use tokio::net::TcpListener;
use tokio::process::Command;
use tower::ServiceBuilder;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};

use common::errors::{ApiError, timeout_or_500};
use common::init_tracing;

static PORT: LazyLock<u16> = LazyLock::new(|| {
    std::env::var("PORT")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(3000)
});
static APP_DIR: LazyLock<String> =
    LazyLock::new(|| std::env::var("APP_DIR").unwrap_or_else(|_| "../../app".to_string()));
static TRUNK_BIN: LazyLock<String> =
    LazyLock::new(|| std::env::var("TRUNK_BIN").unwrap_or_else(|_| "trunk".to_string()));

#[derive(Deserialize)]
struct RunPayload {
    code: String,
}

const INDEX_HTML: &str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, user-scalable=no, initial-scale=1.0, maximum-scale=1.0, minimum-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
    <title>Document</title>
</head>
<body>
    <script type="module">
    /*JS_GOES_HERE*/
    /*INIT_GOES_HERE*/
    </script>
</body>
</html>
"#;

async fn run(Query(body): Query<RunPayload>) -> Result<Html<String>, ApiError> {
    if body.code.is_empty() {
        return Err(ApiError::NoBody);
    }

    #[cfg(feature = "simulate-delay")]
    {
        let delay: u64 = std::env::var("SIMULATE_DELAY_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);
        info!(delay, "simulating cold start delay");
        tokio::time::sleep(Duration::from_secs(delay)).await;
    }

    let app_dir = fs::canonicalize(&*APP_DIR).await.map_err(|e| {
        error!(?e, "failed to canonicalize app_dir path");
        ApiError::IoError(e)
    })?;

    fs::write(app_dir.join("src/main.rs"), &body.code)
        .await
        .map_err(|e| {
            error!(?e, "failed to write main.rs");
            ApiError::IoError(e)
        })?;

    let mut cmd = Command::new(&*TRUNK_BIN);
    let cmd = cmd
        .arg("--config")
        .arg(app_dir.join("Trunk.toml"))
        .arg("build")
        .kill_on_drop(true);
    debug!(?cmd, "running command");

    let output = cmd.output().await.map_err(|e| {
        error!(?e, "running trunk failed");
        ApiError::IoError(e)
    })?;

    if !output.status.success() {
        return Ok(Html(String::from_utf8_lossy(&output.stderr).to_string()));
    }

    let dist = app_dir.join("dist");
    let js = fs::read_to_string(dist.join("app.js")).await.map_err(|e| {
        error!(?e, "failed to read app.js");
        ApiError::IoError(e)
    })?;
    let wasm = fs::read(dist.join("app_bg.wasm")).await.map_err(|e| {
        error!(?e, "failed to read app_bg.wasm");
        ApiError::IoError(e)
    })?;

    debug!(wasm_bytes = wasm.len(), "compilation successful");

    let init_fn = js
        .split("export default")
        .nth(1)
        .and_then(|it| it.trim().strip_suffix(";"))
        .or_else(|| {
            js.lines().rev().find_map(|line| {
                let line = line.trim();
                let line = line.strip_prefix("export")?;
                let line = line.trim().strip_prefix('{')?;
                let line = line.trim().strip_suffix("};")?;
                line.split(',').find_map(|part| {
                    let part = part.trim();
                    part.strip_suffix("as default").map(|name| name.trim())
                })
            })
        });

    match init_fn {
        Some(init_fn) => {
            let index_html = INDEX_HTML.replace("/*JS_GOES_HERE*/", &js);
            let init = format!("{}((new Int8Array({:?})).buffer)", init_fn, wasm);
            let index_html = index_html.replace("/*INIT_GOES_HERE*/", &init);
            Ok(Html(index_html))
        }
        None => Err(ApiError::Unknown(anyhow!(
            "failed to find init function as default export in js"
        ))),
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    let app_dir = &*APP_DIR;
    let trunk_path = &*TRUNK_BIN;
    debug!(?app_dir);

    let trunk_version = Command::new(trunk_path)
        .arg("--version")
        .output()
        .await
        .map(|v| String::from_utf8_lossy(&v.stdout).trim().to_string())
        .unwrap_or_else(|_| "failed to get trunk version".to_string());
    debug!(trunk_bin_path = ?trunk_path, trunk_version = ?trunk_version);

    let api = Router::new()
        .route("/run", get(run))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(timeout_or_500))
                .timeout(Duration::from_secs(60)),
        )
        .layer(GlobalConcurrencyLimitLayer::new(1))
        .layer(TraceLayer::new_for_http());

    let app = Router::new()
        .nest("/api", api)
        .layer(CorsLayer::permissive());

    let addr = format!("0.0.0.0:{}", *PORT);
    let listener = TcpListener::bind(&addr).await.unwrap();
    info!("Server running on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
