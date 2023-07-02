use std::ops::Deref;

use clap::{Parser, ValueEnum};
use miette::miette;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short = 'X', long = "request", value_enum, default_value_t=RequestMethod::Get)]
    #[arg(help = "Use request method")]
    pub request: RequestMethod,
    #[arg(long = "info")]
    #[arg(help = "Display port and authentication")]
    pub info: bool,
    #[arg(required_unless_present = "info")]
    #[arg(help = "The LCU resource path e.g. '/lol-summoner/v1/current-summoner'")]
    pub path: Option<String>,
    #[arg(long = "json")]
    #[arg(help = "Send JSON data")]
    pub json: Option<String>,
}

pub enum Protocol {
    Rest(String),
    WSS(String),
}

impl Cli {
    fn path_parts(&self) -> Vec<String> {
        let path = self.path.clone().unwrap();
        path.as_str().split("//").map(str::to_string).collect()
    }

    fn parsed_path(&self) -> Result<String, miette::ErrReport> {
        match self.path_parts().last() {
            Some(path) => Ok(path.to_string()),
            None => ewrap(self.path.clone().unwrap().as_str(), "when processing path"),
        }
    }

    pub fn protocol(&self) -> Result<Protocol, miette::ErrReport> {
        let path = self.parsed_path()?;
        match self.path_parts().first() {
            Some(protocol) => match protocol.deref() {
                "wss:" => Ok(Protocol::WSS(path)),
                _ => Ok(Protocol::Rest(path)),
            },
            None => ewrap(&path, "when processing protocol"),
        }
    }

    pub fn rest_path(&self) -> Result<String, miette::ErrReport> {
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

    pub fn request_body(&self) -> Result<Option<Value>, miette::ErrReport> {
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
