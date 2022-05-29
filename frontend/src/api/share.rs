use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use anyhow::{anyhow, Context, Result};
use gloo_net::http::{Request, Response};
use serde_json::Value;

#[cfg(feature = "emulator")]
const FIRESTORE_URL: &str =
    "http://localhost:8080/v1/projects/pastify-app/databases/(default)/documents/pastes";

#[cfg(not(feature = "emulator"))]
const FIRESTORE_URL: &str =
    "https://firestore.googleapis.com/v1/projects/pastify-app/databases/(default)/documents/pastes";

#[derive(Debug, thiserror::Error, Deserialize)]
#[error("{message}")]
pub struct FirestoreError {
    pub code: u16,
    pub message: String,
    pub status: String,
}

impl FirestoreError {
    async fn anyhow_from_response<V>(resp: &Response) -> Result<V>  {
        let mut error = resp.json::<Value>().await?;
        let error = error.get_mut(ERROR).ok_or_else(|| anyhow!("invalid schema returned by firestore"))?;
        let error = error.take();
        let error = serde_json::from_value::<FirestoreError>(error).context("invalid schema for error field returned by firestore")?;
        Err(error.into())
    }
}

#[derive(Serialize, Deserialize)]
pub struct PasteFields {
    content: HashMap<String, String>,
    #[serde(rename = "createdBy")]
    created_by: HashMap<String, Option<String>>,
}

const STRING_VALUE: &str = "stringValue";
const NULL_VALUE: &str = "nullValue";
const ERROR: &str = "error";

impl PasteFields {
    pub fn content(&self) -> &str {
        self.content.get(STRING_VALUE).unwrap().as_str()
    }

    pub fn into_content(mut self) -> String {
        self.content.remove(STRING_VALUE).unwrap()
    }

    pub fn created_by(&self) -> Option<&str> {
        self.created_by
            .get(STRING_VALUE)
            .map(|it| it.as_ref().unwrap().as_str())
    }
}

impl fmt::Debug for PasteFields {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Fields")
            .field("content", &self.content())
            .field("created_by", &format!("{:?}", self.created_by()))
            .finish()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PasteDocument {
    #[serde(skip_serializing)]
    name: String,
    pub fields: PasteFields,
    #[serde(rename = "createTime")]
    #[serde(skip_serializing)]
    pub create_time: String,
    #[serde(rename = "updateTime")]
    #[serde(skip_serializing)]
    update_time: String,
}

impl PasteDocument {
    pub fn id(&self) -> String {
        self.name.replace(
            "projects/pastify-app/databases/(default)/documents/pastes/",
            "",
        )
    }
}

pub async fn get(id: &str) -> Result<PasteDocument> {
    let resp = Request::get(&format!("{}/{}", FIRESTORE_URL, id)).send().await?;
    if resp.status() == 200 {
        Ok(resp.json().await?)
    } else {
        return FirestoreError::anyhow_from_response(&resp).await;
    }
}

pub async fn create(content: &str) -> Result<PasteDocument> {
    let doc = PasteDocument {
        name: String::new(),
        fields: PasteFields {
            content: {
                let mut map = HashMap::with_capacity(1);
                map.insert(STRING_VALUE.to_string(), content.to_string());
                map
            },
            created_by: {
                let mut map = HashMap::with_capacity(1);
                map.insert(NULL_VALUE.to_string(), None);
                map
            },
        },
        create_time: String::new(),
        update_time: String::new(),
    };

    let resp = Request::post(FIRESTORE_URL)
        .json(&doc)?
        .send()
        .await?;

    if (200..300).contains(&resp.status()) {
        Ok(resp.json().await?)
    } else {
        return FirestoreError::anyhow_from_response(&resp).await;
    }
}
