use clap::{Args, Parser, Subcommand, ValueEnum};
use futures_util::StreamExt;
use irelia::rest::LCUClient;
use irelia::ws::{EventType, LCUWebSocket};
use miette::miette;
use miette::Result;
use serde_json::Value;

use crate::ws::{MinimalMessage, VerboseMessage, WebSocketMessage};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    // Send a web request to the LCU
    Request(RequestArgs),
    // Subscribe and listen to a WebSocket event
    Subscribe(SubscribeArgs),
    // Display the current port and authorization
    Info,
}

#[derive(Args)]
pub struct RequestArgs {
    #[arg(help = "The resource path e.g. //lol-summoner/v1/current-summoner")]
    path: String,
    #[arg(
        short = 'X',
        long = "request",
        value_enum,
        default_value_t=RequestMethod::Get
    )]
    request: RequestMethod,
    #[arg(long = "json", help = "Send JSON data")]
    json: Option<String>,
}

impl RequestArgs {
    pub async fn execute(&self, client: &'_ LCUClient<'_>) -> Result<(), miette::ErrReport> {
        let body = self.request_body()?;
        let path = self.rest_path()?;
        let res =
            super::rest::send_request(&client, self.request.clone(), path.as_str(), body).await;
        let message = match res {
            Ok(value) => {
                let value = value.map_or(serde_json::json!("undefined"), |v| v);
                colored_json::to_colored_json_auto(&value).map_err(|e| {
                    ewrap::<()>(&e.to_string(), "when pretty printing JSON").unwrap_err()
                })
            }
            Err(e) => ewrap(&e.to_string(), "when processing LCU request"),
        };

        let m = message?;
        println!("{}", m);
        Ok(())
    }

    fn path_parts(&self) -> Vec<String> {
        let path = self.path.clone();
        path.as_str()
            .split("//")
            .map(str::to_string)
            .collect::<Vec<String>>()
    }

    fn parsed_path(&self) -> Result<String, miette::ErrReport> {
        match self.path_parts().last() {
            Some(path) => Ok(path.to_string()),
            None => ewrap(self.path.clone().as_str(), "when processing path"),
        }
    }

    fn rest_path(&self) -> Result<String, miette::ErrReport> {
        let path = self.parsed_path()?;
        match path.chars().next() {
            Some(char) => {
                if char.eq(&'/') {
                    Ok(path)
                } else {
                    Ok(format!("/{path}"))
                }
            }
            None => ewrap(&path, "when processing rest path"),
        }
    }

    fn request_body(&self) -> Result<Option<Value>, miette::ErrReport> {
        match &self.json {
            Some(string) => {
                let value = serde_json::from_str::<Value>(string.as_str());
                match value {
                    Ok(value) => Ok(Some(value)),
                    Err(_) => ewrap("Bad JSON input", "when serializing body"),
                }
            }
            None => Ok(None),
        }
    }
}

#[derive(Args)]
pub struct SubscribeArgs {
    #[arg(help = "The WebSocket event e.g. OnJsonApiEvent")]
    event: String,
    #[arg(long = "filter", help = "Filter event URI (case-sensitive)")]
    filter: Option<String>,
    #[arg(short = 'v', long = "verbose", help = "Print detailed data")]
    verbose: bool,
}

impl SubscribeArgs {
    pub async fn execute(&self) -> Result<()> {
        // Listen to WebSocket event
        match LCUWebSocket::new().await {
            Ok(mut ws) => {
                let event = match self.event.as_str() {
                    "OnJsonApiEvent" => Ok(EventType::OnJsonApiEvent),
                    "OnLcdEvent" => Ok(EventType::OnLcdEvent),
                    _ => ewrap(&self.event, "when processing websocket event"),
                };
                ws.subscribe(event?);
                while let Some(event) = ws.next().await {
                    let Ok(value) = event else {
                        return ewrap(&event.err().unwrap().to_string(), "when processing websocket");
                    };
                    let wsm = WebSocketMessage::from(&value);
                    // I don't know about this... :c
                    match &self.verbose {
                        true => match &self.filter {
                            Some(name) => {
                                if wsm.uri.contains(name) {
                                    println!("{}", VerboseMessage::from(&wsm))
                                }
                            }
                            None => println!("{}", VerboseMessage::from(&wsm)),
                        },
                        false => match &self.filter {
                            Some(name) => {
                                if wsm.uri.contains(name) {
                                    println!("{}", MinimalMessage::from(&wsm));
                                }
                            }
                            None => println!("{}", MinimalMessage::from(&wsm)),
                        },
                    };
                }
                Ok(())
            }
            Err(e) => {
                return ewrap(&e.to_string(), "when creating websocket");
            }
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
pub enum RequestMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

pub fn ewrap<T>(err: &str, msg: &'static str) -> Result<T, miette::ErrReport> {
    Err((miette!("{}", err)).wrap_err(msg))
}
