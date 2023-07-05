use clap::Parser;
use irelia::{rest::LCUClient, RequestClient};
use miette::Result;

use cli::{ewrap, Cli, Commands};

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

    let status = match cli.command {
        // Display port and authorization only
        Commands::Info => {
            println!("{}", ClientInfo::from(&client));
        }
        // Subscribe to WebSocket event and listen for data
        Commands::Subscribe(args) => args.execute().await?,
        Commands::Request(args) => args.execute(&client).await?,
    };
    Ok(status)
}
