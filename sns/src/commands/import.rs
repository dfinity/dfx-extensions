//! Code for the command line `dfx sns import`
// use crate::lib::info::replica_rev;
// use crate::lib::project::import::import_canister_definitions;
// use crate::lib::project::network_mappings::get_network_mappings;
use crate::lib::call_bundled::replica_rev;
use crate::lib::project::{get_network_mappings, import_canister_definitions};
use dfx_core::config::model::dfinity::Config;

use clap::Parser;
use tokio::runtime::Runtime;

/// Imports the sns canisters
#[derive(Parser)]
pub struct SnsImportOpts {
    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[clap(long, default_value = "ic=mainnet", multiple_occurrences(true))]
    network_mapping: Vec<String>,
}

/// Executes the command line `dfx sns import`.
pub fn exec(opts: SnsImportOpts) -> anyhow::Result<()> {
    let config = Config::from_current_dir()?;
    let mut config = config.unwrap();
    // let mut config = config.as_ref().clone();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;

    let runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_commit = std::env::var("DFX_IC_COMMIT").unwrap_or_else(|_| replica_rev().unwrap());
    let their_dfx_json_location =
        format!("https://raw.githubusercontent.com/dfinity/ic/{ic_commit}/rs/sns/cli/dfx.json");
    let logger = slog::Logger::root(slog::Discard, slog::o!());

    runtime.block_on(import_canister_definitions(
        &logger,
        &mut config,
        &their_dfx_json_location,
        None,
        None,
        &network_mappings,
    ));
    Ok(())
}

// TODO
