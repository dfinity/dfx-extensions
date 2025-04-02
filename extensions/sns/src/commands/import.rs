//! Code for the command line `dfx sns import`
use std::path::Path;

use dfx_core::config::model::dfinity::Config;
use dfx_extensions_utils::dependencies::dfx::NNS_SNS_REPLICA_REV;
use dfx_extensions_utils::{get_network_mappings, import_canister_definitions, new_logger};

use clap::Parser;
use dfx_core::config::cache::get_version_from_cache_path;
use dfx_core::extension::manager::ExtensionManager;

/// Imports the sns canisters
#[derive(Parser)]
pub struct SnsImportOpts {
    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[arg(long, default_value = "ic=mainnet", action = clap::ArgAction::Append)]
    network_mapping: Vec<String>,
}

/// Executes the command line `dfx sns import`.
pub async fn exec(opts: SnsImportOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let version = get_version_from_cache_path(dfx_cache_path)?;
    let extension_manager = ExtensionManager::new(&version)?;
    let config = Config::from_current_dir(Some(&extension_manager))?;
    if config.is_none() {
        anyhow::bail!(crate::errors::DFXJSON_NOT_FOUND);
    }
    let mut config = config.unwrap();
    let logger = new_logger();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;

    let ic_commit = std::env::var("DFX_IC_COMMIT").unwrap_or(NNS_SNS_REPLICA_REV.to_string());
    let their_dfx_json_location =
        format!("https://raw.githubusercontent.com/dfinity/ic/{ic_commit}/rs/sns/cli/dfx.json");
    import_canister_definitions(
        &logger,
        &mut config,
        &their_dfx_json_location,
        None,
        None,
        &network_mappings,
    )
    .await?;
    Ok(())
}
