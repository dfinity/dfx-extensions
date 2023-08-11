//! Code for decentralizing dapps
#![warn(clippy::missing_docs_in_private_items)]
pub mod commands;
pub mod create_config;
pub mod deploy;
mod errors;
pub mod validate_config;

/// The default location of an SNS configuration file.
pub const CONFIG_FILE_NAME: &str = "sns.yml";

use std::path::PathBuf;

// #![warn(clippy::missing_docs_in_private_items)]
use crate::{
    commands::config::SnsConfigOpts, commands::deploy::DeployOpts,
    commands::download::SnsDownloadOpts, commands::import::SnsImportOpts,
    commands::prepare_canisters::SnsPrepareCanistersOpts,
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
    #[arg(long, env = "DFX_CACHE_PATH", global = true)]
    dfx_cache_path: Option<PathBuf>,
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
    /// Subcommand for preparing dapp canister(s) for 1-proposal SNS creation
    #[command()]
    PrepareCanisters(SnsPrepareCanistersOpts),
}

/// Executes `dfx sns` and its subcommands.
fn main() -> anyhow::Result<()> {
    let opts = SnsOpts::parse();
    let dfx_cache_path = &opts.dfx_cache_path.ok_or_else(|| {
        anyhow::Error::msg(
            "Missing path to dfx cache. Pass it as CLI argument: `--dfx-cache-path=PATH`",
        )
    })?;

    match opts.subcmd {
        SubCommand::Config(v) => commands::config::exec(v, &dfx_cache_path),
        SubCommand::Import(v) => commands::import::exec(v, &dfx_cache_path),
        SubCommand::Deploy(v) => commands::deploy::exec(v, &dfx_cache_path),
        SubCommand::Download(v) => commands::download::exec(v, &dfx_cache_path),
        SubCommand::PrepareCanisters(v) => commands::prepare_canisters::exec(v, &dfx_cache_path),
    }?;
    Ok(())
}
