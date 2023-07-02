use clap::{Parser, ValueEnum};
use miette::miette;

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
