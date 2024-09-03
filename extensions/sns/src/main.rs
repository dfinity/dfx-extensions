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
    add_sns_wasm_for_tests, deploy_testflight,
    init_config_file::{self, InitConfigFileArgs},
    list::{self, ListArgs},
    neuron_id_to_candid_subaccount::{self, NeuronIdToCandidSubaccountArgs},
    prepare_canisters::{self, PrepareCanistersArgs},
    propose::{self, ProposeArgs},
    AddSnsWasmForTestsArgs, DeployTestflightArgs, SubCommand as SnsLibSubCommand,
};
mod utils;

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

/// Initialize, deploy and interact with an SNS.
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
    /// Converts a Neuron ID to a candid subaccount blob suitable for use in
    /// the `manage_neuron` method on SNS Governance.
    #[command()]
    NeuronIdToCandidSubaccount(NeuronIdToCandidSubaccountArgs),
    /// Add a wasms for one of the SNS canisters, skipping the NNS proposal,
    /// for tests.
    #[command(hide(true))]
    AddSnsWasmForTests(AddSnsWasmForTestsArgs),
    /// List SNSes
    List(ListArgs),

    /// Subcommand for importing sns API definitions and canister IDs.
    /// This and `Download` are only useful for SNS testflight
    #[command()]
    Import(SnsImportOpts),
    /// Downloads SNS canister versions that are specified in your dfx.json (which probably got there through the `Import` command).
    #[command()]
    Download(SnsDownloadOpts),
}

/// Executes `dfx sns` and its subcommands.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = SnsOpts::parse();

    let subcommand: SnsLibSubCommand = match opts.subcmd {
        SubCommand::DeployTestflight(args) => SnsLibSubCommand::DeployTestflight(args),
        SubCommand::InitConfigFile(args) => SnsLibSubCommand::InitConfigFile(args),
        SubCommand::PrepareCanisters(args) => SnsLibSubCommand::PrepareCanisters(args),
        SubCommand::Propose(args) => SnsLibSubCommand::Propose(args),
        SubCommand::NeuronIdToCandidSubaccount(args) => {
            SnsLibSubCommand::NeuronIdToCandidSubaccount(args)
        }
        SubCommand::AddSnsWasmForTests(args) => SnsLibSubCommand::AddSnsWasmForTests(args),
        SubCommand::List(args) => SnsLibSubCommand::List(args),

        SubCommand::Import(v) => {
            let dfx_cache_path = &opts.dfx_cache_path.ok_or_else(|| {
                anyhow::Error::msg(
                    "Missing path to dfx cache. Pass it as CLI argument: `--dfx-cache-path=PATH`",
                )
            })?;
            return commands::import::exec(v, dfx_cache_path);
        }
        SubCommand::Download(v) => {
            let dfx_cache_path = &opts.dfx_cache_path.ok_or_else(|| {
                anyhow::Error::msg(
                    "Missing path to dfx cache. Pass it as CLI argument: `--dfx-cache-path=PATH`",
                )
            })?;
            return commands::download::exec(v, dfx_cache_path);
        }
    };

    let agent = utils::get_mainnet_agent()?;

    match subcommand {
        SnsLibSubCommand::DeployTestflight(args) => deploy_testflight(args),
        SnsLibSubCommand::InitConfigFile(args) => init_config_file::exec(args),
        SnsLibSubCommand::PrepareCanisters(args) => prepare_canisters::exec(args),
        SnsLibSubCommand::Propose(args) => propose::exec(args),
        SnsLibSubCommand::NeuronIdToCandidSubaccount(args) => {
            neuron_id_to_candid_subaccount::exec(args)
        }
        SnsLibSubCommand::AddSnsWasmForTests(args) => add_sns_wasm_for_tests(args),
        SnsLibSubCommand::List(args) => list::exec(args, &agent).await,
    }
}

#[test]
fn verify_extension_manifest() {
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!("Project root: {:?}", project_root);

    let dest_path = project_root.join("extension.json");

    dfx_extensions_utils::manifest::verify_extension_manifest::<SubCommand>(dest_path.as_path())
        .unwrap();
}
