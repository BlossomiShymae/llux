use clap::{Parser, ValueEnum};
use colored_json::to_colored_json_auto;
use irelia::{rest::LCUClient, RequestClient};
use miette::{miette, Result};
use serde_json::{json, Value};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short = 'X', long = "request", value_enum, default_value_t=RequestMethod::Get)]
    #[arg(help = "Use request method")]
    request: RequestMethod,
    #[arg(help = "The LCU resource path e.g. '/lol-summoner/v1/current-summoner'")]
    path: String,
    #[arg(long = "json")]
    #[arg(help = "Send JSON data")]
    json: Option<String>,
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

    // Parse the LCU resource path ⚙
    let parts: Vec<&str> = args.path.as_str().split("//").collect();
    let Some(path) = parts.last() else {
        let err = args.path.as_str();
        return Err((miette!("{}", err)).wrap_err("when processing path"));
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
