use crate::api::BACKEND_URL;
use anyhow::Result;
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

#[derive(Debug, PartialEq)]
pub enum Response {
    Render(RunResponse),
    CompileError(String),
}

pub async fn run(value: &str) -> Result<Rc<Response>> {
    let payload = RunPayload {
        main_contents: value,
    };
    let resp = Request::post(&format!("{BACKEND_URL}/run"))
        .body(serde_json::to_string(&payload).unwrap())
        .header("Content-Type", "application/json")
        .send()
        .await?;

    let resp = if resp.status() == 200 {
        let bin = resp.binary().await.unwrap();
        let resp = bson::from_slice::<RunResponse>(&bin)?;
        Response::Render(resp)
    } else {
        let err = resp.text().await?;
        Response::CompileError(err)
    };

    Ok(Rc::new(resp))
}
