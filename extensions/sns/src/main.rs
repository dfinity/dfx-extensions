//! Code for decentralizing dapps
#![warn(clippy::missing_docs_in_private_items)]
pub mod commands;
mod errors;

/// The default location of an SNS configuration file.
pub const CONFIG_FILE_NAME: &str = "sns.yml";

use std::path::PathBuf;

// #![warn(clippy::missing_docs_in_private_items)]
use crate::commands::{download::SnsDownloadOpts, import::SnsImportOpts};

use clap::Parser;
use ic_sns_cli::{
    deploy_testflight,
    init_config_file::{self, InitConfigFileArgs},
    prepare_canisters::{self, PrepareCanistersArgs},
    propose::{self, ProposeArgs},
    DeployTestflightArgs,
};

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
    /// Manage the config file where the initial sns parameters are set.
    #[command()]
    InitConfigFile(InitConfigFileArgs),
    /// Adds or removes NNS root as a controller to canisters controlled by the current dfx identity to prepare for SNS Decentralization.
    /// NNS root must be added as a controller to all canisters that will be controlled by the SNS before the proposal is submitted.
    #[command()]
    PrepareCanisters(PrepareCanistersArgs),
    /// Deploy an sns directly to a subnet, skipping the sns-wasms canister.
    /// The SNS canisters remain controlled by the developer after deployment.
    /// For use in tests only.
    #[command()]
    DeployTestflight(DeployTestflightArgs),
    /// Submit an NNS proposal to create new SNS.
    #[command()]
    Propose(ProposeArgs),

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
        SubCommand::DeployTestflight(args) => {
            deploy_testflight(args);
            Ok(())
        }
        SubCommand::InitConfigFile(args) => {
            init_config_file::exec(args);
            Ok(())
        }
        SubCommand::PrepareCanisters(args) => {
            prepare_canisters::exec(args);
            Ok(())
        }
        SubCommand::Propose(args) => {
            propose::exec(args);
            Ok(())
        }

        SubCommand::Import(v) => {
            let dfx_cache_path = &opts.dfx_cache_path.ok_or_else(|| {
                anyhow::Error::msg(
                    "Missing path to dfx cache. Pass it as CLI argument: `--dfx-cache-path=PATH`",
                )
            })?;
            commands::import::exec(v, dfx_cache_path)
        }
        SubCommand::Download(v) => {
            let dfx_cache_path = &opts.dfx_cache_path.ok_or_else(|| {
                anyhow::Error::msg(
                    "Missing path to dfx cache. Pass it as CLI argument: `--dfx-cache-path=PATH`",
                )
            })?;
            commands::download::exec(v, dfx_cache_path)
        }
    }?;
    Ok(())
}
