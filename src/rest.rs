use data_encoding::BASE64;
use irelia::{rest::LCUClient, LCUError};
use owo_colors::OwoColorize;
use serde_json::Value;
use std::{fmt::Display, str};

use crate::cli::RequestMethod;

#[derive(Debug, Clone)]
pub struct ClientInfo {
    host: String,
    auth: String,
}

impl ClientInfo {
    fn decoded_auth(&self) -> String {
        let parts = self.auth.split("Basic ").collect::<Vec<&str>>();
        let auth_token = parts.last().unwrap();
        let decoded_token = BASE64.decode(auth_token.as_bytes()).unwrap();
        String::from_utf8(decoded_token).unwrap()
    }
}

impl From<&LCUClient<'_>> for ClientInfo {
    fn from(value: &LCUClient<'_>) -> Self {
        Self {
            host: value.url().to_string(),
            auth: value.auth_header().to_string(),
        }
    }
}

impl Display for ClientInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}: {}", "host".bright_purple(), self.host)?;
        writeln!(f, "{}: {}", "authorization".bright_purple(), self.auth)?;
        writeln!(
            f,
            "{}: Basic {}",
            "authorization (decoded)".bright_purple(),
            self.decoded_auth().bright_yellow()
        )
    }
}

pub async fn send_request(
    client: &LCUClient<'_>,
    method: RequestMethod,
    path: &str,
    body: Option<Value>,
) -> Result<Option<Value>, LCUError> {
    match method {
        RequestMethod::Get => client.get::<Value>(path).await,
        RequestMethod::Delete => client.delete::<Value>(path).await,
        RequestMethod::Head => client.head::<Value>(path).await,
        RequestMethod::Post => client.post::<Value, Value>(path, body).await,
        RequestMethod::Put => client.put::<Value, Value>(path, body).await,
        RequestMethod::Patch => client.patch::<Value, Value>(path, body).await,
    }
}
