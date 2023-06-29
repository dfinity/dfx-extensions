//! Code for the command line: `dfx nns install`
use std::{path::Path, sync::Arc};

use crate::install_nns::{get_and_check_replica_url, get_with_retries, install_nns};
use dfx_core::{
    config::model::dfinity::{Config, NetworksConfig},
    network::{
        provider::{create_network_descriptor, LocalBindDetermination},
        root_key::fetch_root_key_when_local,
    },
};

use clap::Parser;
use dfx_extensions_utils::{new_logger, webserver_port};
use ic_agent::agent::http_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::Agent;

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
    #[arg(long, action = clap::ArgAction::Append)]
    ledger_accounts: Vec<String>,
}

/// Executes `dfx nns install`.
pub async fn exec(opts: InstallOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let agent = Agent::builder()
        .with_transport(
            ReqwestHttpReplicaV2Transport::create(format!(
                "http://127.0.0.1:{}",
                webserver_port(dfx_cache_path)?
            ))
            .unwrap(),
        )
        .with_identity(ic_agent::identity::AnonymousIdentity)
        .build()?;
    let networks_config = NetworksConfig::new()?;
    let logger = new_logger();

    let config = Config::from_current_dir()?;
    if config.is_none() {
        anyhow::bail!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?");
    }
    let network_descriptor = create_network_descriptor(
        Some(Arc::new(config.unwrap())),
        Arc::new(networks_config.clone()),
        Some("local".to_string()),
        Some(logger.clone()),
        LocalBindDetermination::ApplyRunningWebserverPort, // TODO: is this the correct choice?
    )?;

    // Wait for the server to be ready...
    let nns_url = get_and_check_replica_url(&network_descriptor, &logger)?;
    get_with_retries(&nns_url).await?;

    fetch_root_key_when_local(&agent, &network_descriptor).await?;

    let ic_nns_init_path = dfx_cache_path.join("ic-nns-init");

    install_nns(
        &agent,
        &network_descriptor,
        &networks_config,
        &dfx_cache_path,
        &ic_nns_init_path,
        &opts.ledger_accounts,
        &logger,
    )
    .await
}
