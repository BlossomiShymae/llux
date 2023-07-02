use clap::Parser;
use colored_json::to_colored_json_auto;
use futures_util::StreamExt;
use irelia::{
    rest::LCUClient,
    ws::{EventType, LCUWebSocket},
    RequestClient,
};
use miette::{miette, Result};
use serde_json::json;

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
    let cli = Cli::parse();
    let r_client = RequestClient::new();
    let client = LCUClient::new(&r_client);
    let Ok(client) = client else {
        let err = client.err().unwrap().to_string();
        return ewrap(&err, "when connecting to the LCU");
    };

    // Display port and auth only
    if cli.info {
        println!("{}", ClientInfo::from(&client));
        return Ok(());
    }

    let protocol = cli.protocol()?;
    if let cli::Protocol::WSS(_event) = protocol {
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
    };

    // Send request to the LCU 💜
    let body = cli.request_body()?;
    let path = cli.rest_path()?;
    let res = rest::send_request(&client, cli.request, path.as_str(), body).await;
    let message = match res {
        Ok(value) => {
            let value = value.map_or(json!("undefined"), |v| v);
            to_colored_json_auto(&value)
                .map_err(|e| ewrap::<()>(&e.to_string(), "when pretty printing JSON").unwrap_err())
        }
        Err(e) => ewrap(&e.to_string(), "when processing LCU request"),
    };

    // End program 🌮
    let m = message?;
    println!("{}", m);
    Ok(())
}
