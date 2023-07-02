use clap::Parser;
use colored_json::to_colored_json_auto;
use data_encoding::BASE64;
use futures_util::StreamExt;
use irelia::{
    rest::LCUClient,
    ws::{EventType, LCUWebSocket},
    RequestClient,
};
use miette::{miette, Result};
use owo_colors::OwoColorize;
use serde_json::{json, Value};
use std::{ops::Deref, str};

use cli::{Cli, RequestMethod};

pub mod cli;
pub mod ws;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(windows)]
    let _enabled = colored_json::enable_ansi_support();

    // Start up the LCU client 🚀
    let args = Cli::parse();
    let r_client = RequestClient::new();
    let client = LCUClient::new(&r_client);
    let Ok(client) = client else {
        let err = client.err().unwrap().to_string();
        return Err((miette!("{}", err)).wrap_err("when connecting to LCU"));
    };

    // Display port and auth only
    if args.info {
        println!("{}: {}", "host".bright_purple(), client.url());
        println!(
            "{}: {}",
            "authorization".bright_purple(),
            client.auth_header()
        );
        let auth: Vec<&str> = client.auth_header().split("Basic ").collect();
        let auth = auth.last().unwrap();
        let decoded = BASE64.decode(auth.as_bytes()).unwrap();
        let decoded = str::from_utf8(decoded.as_slice()).unwrap();
        println!(
            "{}: Basic {}",
            "authorization (decoded)".bright_purple(),
            decoded.bright_yellow().on_black()
        );
        return Ok(());
    }

    // Parse the LCU resource path ⚙
    let path = args.path.unwrap();
    let parts: Vec<&str> = path.as_str().split("//").collect();
    let Some(path) = parts.last() else {
        let err = path.as_str();
        return Err((miette!("{}", err)).wrap_err("when processing path"));
    };
    let Some(event) = parts.first() else {
        let err = path;
        return Err((miette!("{}", err)).wrap_err("when processing path"));
    };
    match event.deref().deref() {
        "wss:" => {
            // Listen to WebSocket event
            match LCUWebSocket::new().await {
                Ok(mut ws) => {
                    ws.subscribe(EventType::OnJsonApiEvent);
                    while let Some(event) = ws.next().await {
                        let Ok(value) = event else {
                            return Err((miette!("{}", event.err().unwrap().to_string())).wrap_err("when processing websocket"));
                        };
                        let wsm = ws::WebSocketMessage::from(&value);
                        let message = to_colored_json_auto(&json!(wsm)).unwrap();
                        println!("{}", message);
                    }
                }
                Err(e) => {
                    return Err(miette!("{}", e.to_string()).wrap_err("when creating websocket"));
                }
            }
        }
        &_ => (),
    };
    let path = format!("/{path}");

    // Serialize body for potential use 🖨
    let body = {
        match args.json {
            Some(json_string) => {
                let value: Result<Value, serde_json::Error> =
                    serde_json::from_str::<Value>(json_string.as_str());
                match value {
                    Ok(value) => Ok(Some(value)),
                    Err(_) => Err((miette!("Bad JSON input")).wrap_err("when serializing body")),
                }
            }
            None => Ok(None),
        }
    };
    let Ok(body) = body else {
        return Err(body.err().unwrap().into());
    };

    // Send request to the LCU 💜
    let path = path.as_str();
    let res = match args.request {
        RequestMethod::Get => client.get::<Value>(path).await,
        RequestMethod::Delete => client.delete::<Value>(path).await,
        RequestMethod::Head => client.head::<Value>(path).await,
        RequestMethod::Post => client.post::<Value, Value>(path, body).await,
        RequestMethod::Put => client.put::<Value, Value>(path, body).await,
        RequestMethod::Patch => client.patch::<Value, Value>(path, body).await,
    };
    let message = match res {
        Ok(value) => {
            let value = value.map_or(json!("undefined"), |v| v);
            to_colored_json_auto(&value)
                .map_err(|e| (miette!("{}", e.to_string())).wrap_err("when pretty printing JSON"))
        }
        Err(e) => Err(miette!("{}", e.to_string()).wrap_err("when processing LCU request")),
    };

    // End program 🌮
    let Ok(m) = message else {
        return Err(message.err().unwrap().into());
    };
    println!("{}", m);
    Ok(())
}
