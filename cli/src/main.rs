use anyhow::Result;
use clap::Parser;

mod commands;

fn main() -> Result<()> {
    let cli = commands::Cli::parse();
    commands::run(cli)
}
