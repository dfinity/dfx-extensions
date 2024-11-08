//! Code for the command line: `dfx nns install`
use crate::install_nns::{get_and_check_replica_url, get_with_retries, install_nns};
use clap::Parser;
use dfx_core::DfxInterfaceBuilder;
use dfx_extensions_utils::new_logger;
use std::path::Path;

/// Installs the NNS canisters, Internet Identity and the NNS frontend dapp
///
/// - The core network nervous system canisters are nns-registry, nns-governance, nns-ledger, nns-root, nns-cycles-minting,
///   nns-lifeline, nns-genesis-token and nns-sns-wasm.
///   Source code is at <https://github.com/dfinity/ic/tree/master/rs/nns#network-nervous-system-nns>.
///
///
/// - internet_identity is a login service.
///   Source code is at <https://github.com/dfinity/internet-identity>.
///   This frontend is typically served at: <http://qaa6y-5yaaa-aaaaa-aaafa-cai.localhost:8080>.
///
/// - nns-dapp is a voting app and wallet. Source code is at <https://github.com/dfinity/nns-dapp>.
///   This frontend is typically served at: <http://qhbym-qaaaa-aaaaa-aaafq-cai.localhost:8080>.
#[derive(Parser)]
#[clap(about)]
pub struct InstallOpts {
    /// Initialize ledger canister with these test accounts
    #[arg(long, action = clap::ArgAction::Append, num_args = 0..)]
    ledger_accounts: Vec<String>,
}

/// Executes `dfx nns install`.
pub async fn exec(opts: InstallOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let dfx = DfxInterfaceBuilder::new()
        .anonymous()
        .with_extension_manager_from_cache_path(dfx_cache_path)?
        .build()
        .await?;
    let mut network_descriptor = dfx.network_descriptor().clone();
    if let Some(ref mut local_server_descriptor) = &mut network_descriptor.local_server_descriptor {
        local_server_descriptor.load_settings_digest()?;
    }

    let logger = new_logger();

    let config = dfx.config();
    if config.is_none() {
        anyhow::bail!(crate::errors::DFXJSON_NOT_FOUND);
    }

    // Wait for the server to be ready...
    let nns_url = get_and_check_replica_url(&network_descriptor, &logger)?;
    get_with_retries(&nns_url).await?;

    install_nns(
        dfx.agent(),
        &network_descriptor,
        dfx.networks_config(),
        dfx_cache_path,
        &opts.ledger_accounts,
        &logger,
    )
    .await
}
