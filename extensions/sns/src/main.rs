//! Code for decentralizing dapps
#![warn(clippy::missing_docs_in_private_items)]
pub mod commands;
pub mod create_config;
pub mod deploy;
pub mod validate_config;

/// The default location of an SNS configuration file.
pub const CONFIG_FILE_NAME: &str = "sns.yml";

use std::{path::PathBuf, str::FromStr};

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

    // global args have to be wrapped with Option for now: https://github.com/clap-rs/clap/issues/1546
    /// Path to cache of DFX which executed this extension.
    #[arg(long, global = true)]
    dfx_cache_path: Option<String>,
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
    let dfx_cache_path = PathBuf::from_str(&opts.dfx_cache_path.unwrap()).unwrap();
    match opts.subcmd {
        SubCommand::Config(v) => commands::config::exec(v, &dfx_cache_path),
        SubCommand::Import(v) => commands::import::exec(v, &dfx_cache_path),
        SubCommand::Deploy(v) => commands::deploy::exec(v, &dfx_cache_path),
        SubCommand::Download(v) => commands::download::exec(v, &dfx_cache_path),
    }?;
    Ok(())
}
