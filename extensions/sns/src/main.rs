//! Code for decentralizing dapps
#![warn(clippy::missing_docs_in_private_items)]
pub mod commands;
pub mod create_config;
pub mod deploy;
pub mod validate_config;

/// The default location of an SNS configuration file.
pub const CONFIG_FILE_NAME: &str = "sns.yml";

// #![warn(clippy::missing_docs_in_private_items)]
use crate::{
    commands::config::SnsConfigOpts, commands::deploy::DeployOpts,
    commands::download::SnsDownloadOpts, commands::import::SnsImportOpts,
};

use clap::Parser;

/// Options for `dfx sns`.
#[derive(Parser)]
#[command(name("sns"))]
pub struct SnsOpts {
    /// Arguments and flags for subcommands.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Subcommands of `dfx sns`
#[derive(Parser)]
enum SubCommand {
    /// Subcommands for working with configuration.
    #[command()]
    Config(SnsConfigOpts),
    /// Subcommand for creating an SNS.
    #[command()]
    Deploy(DeployOpts),
    /// Subcommand for importing sns API definitions and canister IDs.
    #[command()]
    Import(SnsImportOpts),
    /// Subcommand for downloading SNS WASMs.
    #[command()]
    Download(SnsDownloadOpts),
}

/// Executes `dfx sns` and its subcommands.
fn main() -> anyhow::Result<()> {
    let opts = SnsOpts::parse();
    match opts.subcmd {
        SubCommand::Config(v) => commands::config::exec(v),
        SubCommand::Import(v) => commands::import::exec(v),
        SubCommand::Deploy(v) => commands::deploy::exec(v),
        SubCommand::Download(v) => commands::download::exec(v),
    }?;
    Ok(())
}
