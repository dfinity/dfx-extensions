//! Code for the command line `dfx nns`.
#![warn(clippy::missing_docs_in_private_items)]

use std::{path::PathBuf, str::FromStr};

use clap::Parser;
use tokio::runtime::Runtime;

mod commands;
mod install_nns;
mod nns_types;

/// Options for `dfx nns` and its subcommands.
#[derive(Parser)]
#[clap(name("nns"))]
pub struct NnsOpts {
    /// `dfx nns` subcommand arguments.
    #[clap(subcommand)]
    subcmd: SubCommand,

    // global args have to be wrapped with Option for now: https://github.com/clap-rs/clap/issues/1546
    /// Path to cache of DFX which executed this extension.
    #[arg(long, global = true)]
    dfx_cache_path: Option<String>,
}

/// Command line options for subcommands of `dfx nns`.
#[derive(Parser)]
enum SubCommand {
    /// Import NNS API definitions and canister IDs.
    Import(commands::import::ImportOpts),
    /// Install an NNS on the local dfx server.
    Install(commands::install::InstallOpts),
}

/// Executes `dfx nns` and its subcommands.
pub fn main() -> anyhow::Result<()> {
    let opts = NnsOpts::parse();
    let dfx_cache_path = PathBuf::from_str(&opts.dfx_cache_path.unwrap()).unwrap();
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::Import(v) => commands::import::exec(v, &dfx_cache_path).await,
            SubCommand::Install(v) => commands::install::exec(v, &dfx_cache_path).await,
        }
    })
}
