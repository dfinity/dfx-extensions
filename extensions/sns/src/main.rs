//! Code for decentralizing dapps
#![warn(clippy::missing_docs_in_private_items)]
pub mod commands;
mod errors;

/// The default location of an SNS configuration file.
pub const CONFIG_FILE_NAME: &str = "sns.yml";

use std::path::PathBuf;

// #![warn(clippy::missing_docs_in_private_items)]
use crate::commands::{download::SnsDownloadOpts, import::SnsImportOpts};

use clap::{ArgGroup, Args, Parser};
use ic_agent::Agent;
use ic_sns_cli::{
    add_sns_wasm_for_tests, deploy_testflight,
    health::{self, HealthArgs},
    init_config_file::{self, InitConfigFileArgs},
    list::{self, ListArgs},
    neuron_id_to_candid_subaccount::{self, NeuronIdToCandidSubaccountArgs},
    prepare_canisters::{self, PrepareCanistersArgs},
    propose::{self, ProposeArgs},
    upgrade_sns_controlled_canister::{self, UpgradeSnsControlledCanisterArgs},
    AddSnsWasmForTestsArgs, DeployTestflightArgs, SubCommand as SnsLibSubCommand,
};
mod utils;

#[derive(Args, Clone, Debug, Default)]
#[clap(
group(ArgGroup::new("network-select").multiple(false)),
)]
pub struct NetworkOpt {
    /// Override the compute network to connect to. By default, the local network is used.
    /// A valid URL (starting with `http:` or `https:`) can be used here, and a special
    /// ephemeral network will be created specifically for this request. E.g.
    /// "http://localhost:12345/" is a valid network name.
    #[arg(long, global(true), group = "network-select")]
    network: Option<String>,

    /// Shorthand for --network=playground.
    /// Borrows short-lived canisters on the real IC network instead of creating normal canisters.
    #[clap(long, global(true), group = "network-select")]
    playground: bool,

    /// Shorthand for --network=ic.
    #[clap(long, global(true), group = "network-select")]
    ic: bool,
}

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

    /// The user identity to run this command as. It contains your principal as well as some things DFX associates with it like the wallet.
    #[arg(long, global = true)]
    identity: Option<String>,

    #[command(flatten)]
    network: NetworkOpt,
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
    /// Report health of SNSes
    Health(HealthArgs),
    /// Uploads a given Wasm to a (newly deployed) store canister and submits a proposal to upgrade
    /// using that Wasm.
    UpgradeSnsControlledCanister(UpgradeSnsControlledCanisterArgs),

    /// Subcommand for importing sns API definitions and canister IDs.
    /// This and `Download` are only useful for SNS testflight
    #[command()]
    Import(SnsImportOpts),
    /// Downloads SNS canister versions that are specified in your dfx.json (which probably got there through the `Import` command).
    #[command()]
    Download(SnsDownloadOpts),
}

impl NetworkOpt {
    pub fn to_network_name(&self) -> Option<String> {
        if self.playground {
            Some("playground".to_string())
        } else if self.ic {
            Some("ic".to_string())
        } else {
            self.network.clone()
        }
    }
}

pub async fn agent(network: NetworkOpt, identity: Option<String>) -> anyhow::Result<Agent> {
    let network = match network.to_network_name() {
        Some(network) => network,
        None => {
            eprintln!(
                "No network specified. Defaulting to the local network. To connect to the mainnet IC instead, try passing `--network=ic`"
            );
            "local".to_string()
        }
    };

    match utils::get_agent(&network, identity.clone()).await {
        Ok(agent) => Ok(agent),
        Err(err) => {
            eprintln!("Failed to get agent due to: {err}. \nFalling back to mainnet agent.");
            Agent::builder()
                .with_url("https://ic0.app/")
                .build()
                .map_err(|e| anyhow::anyhow!(e))
        }
    }
}

/// Executes `dfx sns` and its subcommands.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = SnsOpts::parse();

    // Most of the branches in here convert SubCommand to SnsLibSubCommand.
    // The purpose of this is to allow us to match on SnsLibSubCommand. This causes
    // us to have a compiler error if we add a new subcommand to SnsLibSubCommand
    // and don't have some corresponding handling here.
    // TODO(NNS1-3184): Once we remove `Import` and `Download`, this extra step will no longer be
    // needed as we can directly parse the monorepo's command type and subcommands.
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
        SubCommand::Health(args) => SnsLibSubCommand::Health(args),
        SubCommand::UpgradeSnsControlledCanister(args) => {
            SnsLibSubCommand::UpgradeSnsControlledCanister(args)
        }

        SubCommand::Import(v) => {
            let dfx_cache_path = &opts.dfx_cache_path.ok_or_else(|| {
                anyhow::Error::msg(
                    "Missing path to dfx cache. Pass it as CLI argument: `--dfx-cache-path=PATH`",
                )
            })?;
            return commands::import::exec(v, dfx_cache_path).await;
        }
        SubCommand::Download(v) => {
            return commands::download::exec(v).await;
        }
    };

    match subcommand {
        SnsLibSubCommand::DeployTestflight(args) => deploy_testflight(args),
        SnsLibSubCommand::InitConfigFile(args) => init_config_file::exec(args),
        SnsLibSubCommand::PrepareCanisters(args) => prepare_canisters::exec(args),
        SnsLibSubCommand::Propose(args) => propose::exec(args),
        SnsLibSubCommand::NeuronIdToCandidSubaccount(args) => {
            neuron_id_to_candid_subaccount::exec(args)
        }
        SnsLibSubCommand::AddSnsWasmForTests(args) => add_sns_wasm_for_tests(args),
        SnsLibSubCommand::List(args) => {
            let agent = agent(opts.network, opts.identity).await?;
            list::exec(args, &agent).await
        }
        SnsLibSubCommand::Health(args) => {
            let agent = agent(opts.network, opts.identity).await?;

            health::exec(args, &agent).await
        }
        SnsLibSubCommand::UpgradeSnsControlledCanister(args) => {
            let agent = agent(opts.network, opts.identity).await?;
            match upgrade_sns_controlled_canister::exec(args, &agent).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    anyhow::bail!("{}", err)
                }
            }
        }
        SnsLibSubCommand::RefundAfterSnsControlledCanisterUpgrade(args) => {
            let agent = agent(opts.network, opts.identity).await?;
            match upgrade_sns_controlled_canister::refund(args, &agent).await {
                Ok(_) => Ok(()),
                Err(err) => {
                    anyhow::bail!("{}", err)
                }
            }
        }
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
