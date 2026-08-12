mod archive;
mod cli;
mod mongo;
mod s3;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Backup(args) => mongo::backup_database(args).await?,
        Command::Restore(args) => mongo::restore_database(args).await?,
    }

    Ok(())
}
