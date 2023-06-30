use clap::{Parser, ValueEnum};
use irelia::{rest::LCUClient, LCUError, RequestClient};
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'X', long = "request", value_enum, default_value_t=RequestMethod::Get)]
    #[arg(help = "Use request method")]
    request: RequestMethod,
    #[arg(help = "The LCU resource path e.g. '/lol-summoner/v1/current-summoner'")]
    path: String,
}

#[derive(ValueEnum, Debug, Clone)]
enum RequestMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

#[tokio::main]
async fn main() -> Result<(), LCUError> {
    let args = Cli::parse();
    let client = RequestClient::new();
    let res = match LCUClient::new(&client) {
        Ok(client) => {
            let parts: Vec<&str> = args.path.as_str().split("//").collect();
            let uri = parts.last().unwrap();
            let path = format!("/{uri}");
            match args.request {
                RequestMethod::Get => client.get::<Value>(path.as_str()).await,
                _ => panic!("Oh noes!"),
            }
        }
        Err(e) => Err(e),
    };
    res.map(|v| println!("{:?}", v)).map_err(|e| e)
}
