//! Code for the command line: `dfx nns import`
use std::{collections::BTreeMap, path::Path};

use dfx_core::config::cache::get_version_from_cache_path;
use dfx_core::config::model::canister_id_store::CanisterIds;
use dfx_core::config::model::dfinity::Config;
use dfx_core::extension::manager::ExtensionManager;
use dfx_extensions_utils::{
    dependencies::dfx::NNS_SNS_REPLICA_REV, get_canisters_json_object, get_network_mappings,
    import_canister_definitions, new_logger, set_remote_canister_ids, ImportNetworkMapping,
    NNS_CORE,
};

use clap::Parser;
use slog::{info, Logger};

/// Imports the nns canisters
#[derive(Parser)]
pub struct ImportOpts {
    /// Networks to import canisters ids for.
    ///   --network-mapping <network name in both places>
    ///   --network-mapping <network name here>=<network name in project being imported>
    /// Examples:
    ///   --network-mapping ic
    ///   --network-mapping ic=mainnet
    #[clap(long, default_value = "ic=mainnet", action = clap::ArgAction::Append)]
    network_mapping: Vec<String>,
}

/// Executes `dfx nns import`
pub async fn exec(opts: ImportOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let version = get_version_from_cache_path(dfx_cache_path)?;
    let extension_manager = ExtensionManager::new(&version)?;
    let config = Config::from_current_dir(Some(&extension_manager))?;
    if config.is_none() {
        anyhow::bail!(crate::errors::DFXJSON_NOT_FOUND);
    }
    let mut config = config.unwrap().clone();
    let logger = new_logger();

    let network_mappings = get_network_mappings(&opts.network_mapping)?;
    let ic_commit = std::env::var("DFX_IC_COMMIT").unwrap_or(NNS_SNS_REPLICA_REV.to_string());
    let dfx_url_str = {
        let ic_project = std::env::var("DFX_IC_SRC").unwrap_or_else(|_| {
            format!("https://raw.githubusercontent.com/dfinity/ic/{ic_commit}")
        });
        format!("{ic_project}/rs/nns/dfx.json")
    };
    import_canister_definitions(
        &logger,
        &mut config,
        &dfx_url_str,
        Some("nns-"),
        None,
        &network_mappings,
    )
    .await?;

    set_local_nns_canister_ids(&logger, &mut config)
}

/// Sets local canister IDs
/// The "local" entries at the remote URL are often missing or do not match our NNS installation.
/// Always set the local values per our local NNS deployment.  We have all the information locally.
fn set_local_nns_canister_ids(logger: &Logger, config: &mut Config) -> anyhow::Result<()> {
    let local_canister_ids: CanisterIds = NNS_CORE
        .iter()
        .map(|canister| {
            (
                canister.canister_name.to_string(),
                BTreeMap::from([("local".to_string(), canister.canister_id.to_string())]),
            )
        })
        .collect();
    let local_mappings = [ImportNetworkMapping {
        network_name_in_this_project: "local".to_string(),
        network_name_in_project_being_imported: "local".to_string(),
    }];

    let canisters = get_canisters_json_object(config)?;

    for canister in NNS_CORE {
        // Not all NNS canisters may be listed in the remote dfx.json
        let dfx_canister = canisters
            .get_mut(canister.canister_name)
            .and_then(|canister_entry| canister_entry.as_object_mut());
        // If the canister is in dfx.json, set the local canister ID.
        if let Some(dfx_canister) = dfx_canister {
            set_remote_canister_ids(
                logger,
                canister.canister_name,
                &local_mappings,
                &local_canister_ids,
                dfx_canister,
            )?;
        } else {
            info!(
                logger,
                "{} has no local canister ID.", canister.canister_name
            );
        }
    }
    config.save()?;
    Ok(())
}
