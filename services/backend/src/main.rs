use std::net::SocketAddr;

use axum::routing::{get, post};
use axum::{Json, Router};
use axum::body::{Body};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info};
use reqwest::{Client, ResponseBuilderExt};
use anyhow::Error;
use errors::ApiError;
use lazy_static::lazy_static;
use response::Bson;

use common::{errors, init_tracing};
use common::response;

lazy_static! {
    static ref PORT: u16 = std::env::var("PORT")
        .ok()
        .and_then(|it| it.parse().ok())
        .unwrap_or(3000);

    static ref COMPILER_URL: String = std::env::var("COMPILER_URL")
        .expect("COMPILER_URL must be set");

    static ref CLINET: Client = Client::new();
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

async fn run(Json(body): Json<RunPayload>) -> Result<Response<Body>, ApiError> {
    let client = &*CLINET;
    // let res = client.get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity")
    //     .query(&[("audience", &*COMPILER_URL)])
    //     .header("Metadata-Flavor", "Google")
    //     .send()
    //     .await
    //     .expect("can't authenticate with Google metadata server. something horribly gone wrong");

    // let token = res.text().await.map_err(Error::from)?;
    let mut res = client.post(format!("{}/run", *COMPILER_URL))
        .body(body.main_contents)
        // .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(Error::from)?;

    let mut response = Response::builder();
    response.headers_mut().replace(res.headers_mut());

    let response = response.status(res.status())
        .url(res.url().clone())
        .body(Body::from(res.bytes().await.map_err(Error::from)?)).map_err(Error::from)?;
    Ok(response)
}

async fn hello() -> Bson<RunResponse> {
    Bson(RunResponse {
        index_html: "index_html".to_string(),
        js: "js".to_string(),
        wasm: "wasm".as_bytes().to_vec(),
    })
}


#[tokio::main]
async fn main() {
    init_tracing();

    let api = Router::new()
        .route("/hello", get(hello))
        .route("/run", post(run))
        .layer(TraceLayer::new_for_http());

    let app = Router::new().nest("/api", api);

    let addr = SocketAddr::new("0.0.0.0".parse().unwrap(), *PORT);
    info!("Server running on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
