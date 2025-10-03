use clap::Parser;
use secrets_manager::cli::{Cli, SecretManager};
use anyhow::Result;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let manager = SecretManager::new()?;
    
    manager.handle_command(cli.command)?;
    Ok(())
}
