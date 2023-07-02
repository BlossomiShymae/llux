use clap::Parser;
use colored_json::to_colored_json_auto;
use futures_util::StreamExt;
use irelia::{
    rest::LCUClient,
    ws::{EventType, LCUWebSocket},
    RequestClient,
};
use miette::{miette, Result};
use serde_json::{json, Value};
use std::{ops::Deref, str};

use cli::{ewrap, Cli};

use crate::rest::ClientInfo;

pub mod cli;
pub mod rest;
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
        return ewrap(&err, "when connecting to the LCU");
    };

    // Display port and auth only
    if args.info {
        println!("{}", ClientInfo::from(&client));
        return Ok(());
    }

    // Parse the LCU resource path ⚙
    let path = args.path.unwrap();
    let parts: Vec<&str> = path.as_str().split("//").collect();
    let Some(path) = parts.last() else {
        let err = path.as_str();
        return ewrap(&err, "when processing path");
    };
    let Some(event) = parts.first() else {
        let err = path;
        return ewrap(&err, "when processing path");
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
                    return ewrap(&e.to_string(), "when creating websocket");
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
                    Err(_) => ewrap("Bad JSON input", "when serializing body"),
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
    let res = rest::send_request(&client, args.request, path, body).await;
    let message = match res {
        Ok(value) => {
            let value = value.map_or(json!("undefined"), |v| v);
            to_colored_json_auto(&value)
                .map_err(|e| ewrap::<()>(&e.to_string(), "when pretty printing JSON").unwrap_err())
        }
        Err(e) => ewrap(&e.to_string(), "when processing LCU request"),
    };

    // End program 🌮
    let Ok(m) = message else {
        return Err(message.err().unwrap().into());
    };
    println!("{}", m);
    Ok(())
}
