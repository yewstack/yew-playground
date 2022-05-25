use anyhow::{anyhow, Result};
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use std::rc::Rc;

#[derive(Serialize)]
struct RunPayload<'a> {
    main_contents: &'a str,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct RunResponse {
    pub index_html: String,
    pub js: String,
    pub wasm: Vec<u8>,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    #[serde(rename = "output")]
    Render {
        index_html: String,
        js: String,
        wasm: Vec<u8>,
    },
    CompileError(String),
}

pub async fn run(value: &str) -> Result<Response> {
    let payload = RunPayload {
        main_contents: value,
    };
    let resp = Request::post("/api/run")
        .body(serde_json::to_string(&payload).unwrap())
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let status = resp.status();
    if status == 200 {
        let bin = resp.binary().await.unwrap();
        let resp = bson::from_slice::<Response>(&bin)?;
        Ok(resp)
    } else {
        let err = resp.text().await?;
        Err(anyhow!("{}: {}", status, err))
    }
}
