//! Command line interface for `dfx sns`.
#![warn(clippy::missing_docs_in_private_items)]
use crate::{
    commands::config::SnsConfigOpts, commands::download::SnsDownloadOpts,
    commands::import::SnsImportOpts,
};

use clap::{Parser, Subcommand};

mod config;
mod deploy;
mod download;
mod import;

/// Options for `dfx sns`.
#[derive(Parser)]
// #[clap(name("sns"))]
pub struct SnsOpts {
    /// Arguments and flags for subcommands.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Subcommands of `dfx sns`
#[derive(Subcommand)]
pub enum SubCommand {
    /// Subcommands for working with configuration.
    #[clap(hide(true))]
    Config(SnsConfigOpts),
    /// Subcommand for creating an SNS.
    #[clap(hide(true))]
    Deploy(deploy::DeployOpts),
    /// Subcommand for importing sns API definitions and canister IDs.
    #[clap(hide(true))]
    Import(SnsImportOpts),
    /// Subcommand for downloading SNS WASMs.
    #[clap(hide(true))]
    Download(SnsDownloadOpts),
}

/// Executes `dfx sns` and its subcommands.
pub fn exec(cmd: SubCommand) -> anyhow::Result<()> {
    match cmd {
        SubCommand::Config(v) => config::exec(v),
        SubCommand::Import(v) => import::exec(v),
        SubCommand::Deploy(v) => deploy::exec(v),
        SubCommand::Download(v) => download::exec(v),
    }
}
