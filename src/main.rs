use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use flix_dl::api::{Client, Download};

#[derive(Parser)]
struct Cli {
    #[arg(long, env)]
    vip_token: String,

    #[arg(long)]
    id: u32,

    #[arg(long)]
    path: PathBuf,

    #[arg(long)]
    prefetch: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Movie,
    Show,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let client = Client::new("https://flixtor.to", &cli.vip_token)?;

    let download: Box<dyn Download> = match cli.command {
        Command::Movie => Box::new(client.get_movie(cli.id).await?),
        Command::Show => Box::new(client.get_show(cli.id).await?),
    };

    if cli.prefetch {
        download.prefetch(&client).await?;
    }

    download.download(&client, &cli.path).await
}
