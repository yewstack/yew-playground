pub mod errors;
pub mod response;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| {
                "app_compiler=trace,backend=trace,hyper=debug,tower_http=debug".into()
            }),
        ))
        .with(tracing_subscriber::fmt::layer().with_ansi(std::env::var("NO_ANSI_LOG").is_err()))
        .init();
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Output {
        index_html: String,
        js: String,
        wasm: Vec<u8>,
    },
    CompileError(String),
}
