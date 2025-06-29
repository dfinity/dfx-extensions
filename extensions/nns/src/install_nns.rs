//! Implements the `dfx nns install` command, which installs the core NNS canisters, including II and NNS-dapp.
//!
//! Note: `dfx nns` will be a `dfx` plugin, so this code SHOULD NOT depend on NNS code except where extremely inconvenient or absolutely necessary:
//! * Example: Minimise crate dependencies outside the nns modules.
//! * Example: Use `anyhow::Result` not `DfxResult`
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use dfx_core::{error::cli::UserConsent, canister::install_canister_wasm};
use dfx_core::config::model::dfinity::{NetworksConfig, ReplicaSubnetType};
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::identity::CallSender;
use dfx_extensions_utils::dependencies::download_wasms::nns::{
    ICP_INDEX, ICRC1_INDEX, ICRC1_LEDGER, INTERNET_IDENTITY, NNS_DAPP, NNS_LEDGER, SNS_AGGREGATOR,
};
use dfx_extensions_utils::{
    call_extension_bundled_binary, download_nns_wasms, nns_wasm_dir, IcNnsInitCanister,
    SnsCanisterInstallation, StandardCanister, ED25519_TEST_ACCOUNT, NNS_CORE, NNS_CORE_MANUAL,
    NNS_FRONTEND, NNS_SNS_WASM, SECP256K1_TEST_ACCOUNT, SNS_CANISTERS,
};
use ic_sns_cli::{add_sns_wasm_for_tests, AddSnsWasmForTestsArgs};

use anyhow::{anyhow, bail, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::{CandidType, Encode};
use fn_error_context::context;
use futures_util::future::try_join_all;
use ic_agent::export::Principal;
use ic_agent::Agent;
use ic_icp_index::InitArg;
use ic_icrc1_index_ng::{IndexArg, InitArg as IndexInitArg};
use ic_icrc1_ledger::{InitArgsBuilder, LedgerArgument};
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;
use pocket_ic::common::rest::Topology;
use reqwest::Url;
use serde::Serialize;
use slog::Logger;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Init and post_upgrade arguments for NNS frontend dapp.
#[derive(Debug, Eq, PartialEq, CandidType, Serialize)]
pub enum SchemaLabel {
    AccountsInStableMemory,
}
#[derive(Debug, Eq, PartialEq, CandidType, Serialize)]
pub struct CanisterArguments {
    pub args: Vec<(String, String)>,
    pub schema: Option<SchemaLabel>,
}

/// Init and post_upgrade arguments for SNS aggregator.
#[derive(Debug, Eq, PartialEq, CandidType, Serialize)]
pub struct Config {
    pub update_interval_ms: u64,
    pub fast_interval_ms: u64,
}

/// Installs NNS canisters on a local dfx server.
/// # Notes:
///   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different &binary for local development
///   - This won't work with an HSM, because the agent holds a session open
///   - The provider_url is what the agent connects to, and forwards to the replica.
/// # Prerequisites
///   - There must be no canisters already present in the dfx server.
///   - The dfx server must be running as subnet type system; this is set in the local network setting in dfx.json and
///     will normally be different from the production network type, which will most
///     likely be "application".
/// # Errors
/// This will return an error if:
/// - Any of the steps failed to complete.
///
/// # Panics
/// Ideally this should never panic and always return an error on error; while this code is in development reality may differ.
#[context("Failed to install NNS components.")]
pub async fn install_nns(
    agent: &Agent,
    network: &NetworkDescriptor,
    networks_config: &NetworksConfig,
    dfx_cache_path: &Path,
    ledger_accounts: &[String],
    _logger: &Logger,
) -> anyhow::Result<()> {
    eprintln!("Checking out the environment...");
    // Retrieve the PocketIC instance topology.
    let topology = if let Some(descriptor) = &network.local_server_descriptor {
        let endpoint = format!("http://{}/_/topology", descriptor.bind_address);
        let resp = reqwest::get(endpoint).await?;
        if resp.status().is_success() {
            Some(resp.json::<Topology>().await?)
        } else {
            None
        }
    } else {
        None
    };
    // PocketIC has multiple subnets and thus supports default application subnet type.
    if topology.is_none() {
        verify_local_replica_type_is_system(network, networks_config)?;
    }
    verify_nns_canister_ids_are_available(agent).await?;
    let provider_url = get_and_check_provider(network)?;
    let nns_url = provider_url.clone();
    let root_subnet_id = get_subnet_id(agent).await?;
    let sns_subnet_id = topology
        .as_ref()
        .and_then(|topology| topology.get_sns())
        .unwrap_or(root_subnet_id);
    let default_subnet_id = topology
        .as_ref()
        .and_then(|topology| {
            topology
                .get_app_subnets()
                .first()
                .cloned()
                .or_else(|| topology.get_system_subnets().first().cloned())
        })
        .unwrap_or(root_subnet_id);

    eprintln!("Installing the core backend wasm canisters...");
    download_nns_wasms(dfx_cache_path).await?;
    let mut test_accounts = vec![
        ED25519_TEST_ACCOUNT.to_string(),
        SECP256K1_TEST_ACCOUNT.to_string(),
    ];
    test_accounts.extend_from_slice(ledger_accounts);
    let ic_nns_init_opts = IcNnsInitOpts {
        wasm_dir: nns_wasm_dir(dfx_cache_path),
        nns_url: nns_url.to_string(),
        test_accounts,
        sns_subnets: Some(sns_subnet_id.to_string()),
        local_registry_file: network.local_server_descriptor.as_ref().map(|desc| {
            desc.data_dir_by_settings_digest()
                .join("state/replicated_state/registry.proto")
        }),
    };
    ic_nns_init(&ic_nns_init_opts, dfx_cache_path).await?;

    eprintln!("Uploading NNS configuration data...");
    upload_nns_sns_wasms_canister_wasms(dfx_cache_path)?;

    // Install manual backend canisters:
    for IcNnsInitCanister {
        wasm_name,
        canister_name,
        canister_id,
        ..
    } in NNS_CORE_MANUAL
    {
        let local_wasm_path = nns_wasm_dir(dfx_cache_path).join(wasm_name);
        let specified_id = Principal::from_text(canister_id)?;
        let arg = if *canister_id == ICRC1_LEDGER.canister_id {
            let cketh_ledger_args = InitArgsBuilder::for_tests()
                .with_token_symbol("ckETH".to_string())
                .with_token_name("ckETH".to_string())
                .build();
            Some(Encode!(&(LedgerArgument::Init(cketh_ledger_args))).unwrap())
        } else if *canister_id == ICRC1_INDEX.canister_id {
            let cketh_index_args = IndexArg::Init(IndexInitArg {
                ledger_id: Principal::from_str(ICRC1_LEDGER.canister_id).unwrap(),
                retrieve_blocks_from_ledger_interval_seconds: None,
            });
            Some(Encode!(&Some(cketh_index_args)).unwrap())
        } else if *canister_id == ICP_INDEX.canister_id {
            let icp_index_args = InitArg {
                ledger_id: Principal::from_str(NNS_LEDGER.canister_id).unwrap(),
            };
            Some(Encode!(&icp_index_args).unwrap())
        } else {
            None
        };
        let installed_canister_id = install_canister(
            network,
            agent,
            canister_name,
            &local_wasm_path,
            specified_id,
            arg.as_deref(),
        )
        .await?
        .to_text();
        if canister_id != &installed_canister_id {
            bail!("Canister '{canister_name}' was installed at an incorrect canister ID.  Expected '{canister_id}' but got '{installed_canister_id}'.");
        }
    }
    // Install the GUI canisters:
    for StandardCanister {
        wasm_url,
        wasm_name,
        canister_name,
        canister_id,
    } in NNS_FRONTEND
    {
        let local_wasm_path = nns_wasm_dir(dfx_cache_path).join(wasm_name);
        let parsed_wasm_url = Url::parse(wasm_url)
            .with_context(|| format!("Could not parse url for {canister_name} wasm: {wasm_url}"))?;
        download(&parsed_wasm_url, &local_wasm_path).await?;
        let specified_id = Principal::from_text(canister_id)?;
        let arg = if *canister_id == NNS_DAPP.canister_id {
            let nns_dapp_metadata = vec![
                ("API_HOST".to_string(), "http://localhost:8080".to_string()),
                ("CKETH_INDEX_CANISTER_ID".to_string(), ICRC1_INDEX.canister_id.to_string()),
                ("CKETH_LEDGER_CANISTER_ID".to_string(), ICRC1_LEDGER.canister_id.to_string()),
                ("CYCLES_MINTING_CANISTER_ID".to_string(), "rkp4c-7iaaa-aaaaa-aaaca-cai".to_string()),
                ("DFX_NETWORK".to_string(), "local".to_string()),
                ("FEATURE_FLAGS".to_string(), "{\"ENABLE_CKBTC\":false,\"ENABLE_CKTESTBTC\":false,\"ENABLE_HIDE_ZERO_BALANCE\":true,\"ENABLE_VOTING_INDICATION\":true}".to_string()),
                ("FETCH_ROOT_KEY".to_string(), "true".to_string()),
                ("GOVERNANCE_CANISTER_ID".to_string(), "rrkah-fqaaa-aaaaa-aaaaq-cai".to_string()),
                ("HOST".to_string(), "http://localhost:8080".to_string()),
                ("IDENTITY_SERVICE_URL".to_string(), format!("http://{}.localhost:8080", INTERNET_IDENTITY.canister_id)),
                ("INDEX_CANISTER_ID".to_string(), ICP_INDEX.canister_id.to_string()),
                ("LEDGER_CANISTER_ID".to_string(), "ryjl3-tyaaa-aaaaa-aaaba-cai".to_string()),
                ("OWN_CANISTER_ID".to_string(), NNS_DAPP.canister_id.to_string()),
                ("ROBOTS".to_string(), "<meta name=\"robots\" content=\"noindex, nofollow\" />".to_string()),
                ("SNS_AGGREGATOR_URL".to_string(), format!("http://{}.localhost:8080", SNS_AGGREGATOR.canister_id)),
                ("STATIC_HOST".to_string(), "http://localhost:8080".to_string()),
                ("TVL_CANISTER_ID".to_string(), "".to_string()),
                ("WASM_CANISTER_ID".to_string(), "qaa6y-5yaaa-aaaaa-aaafa-cai".to_string())
            ];
            let nns_dapp_init_args = Some(CanisterArguments {
                args: nns_dapp_metadata,
                schema: Some(SchemaLabel::AccountsInStableMemory),
            });
            Some(Encode!(&nns_dapp_init_args).unwrap())
        } else if *canister_id == SNS_AGGREGATOR.canister_id {
            Some(
                Encode!(&Some(Config {
                    update_interval_ms: 1_000,
                    fast_interval_ms: 100,
                }))
                .unwrap(),
            )
        } else {
            None
        };
        let installed_canister_id = install_canister(
            network,
            agent,
            canister_name,
            &local_wasm_path,
            specified_id,
            arg.as_deref(),
        )
        .await?
        .to_text();
        if canister_id != &installed_canister_id {
            bail!("Canister '{canister_name}' was installed at an incorrect canister ID.  Expected '{canister_id}' but got '{installed_canister_id}'.");
        }
    }
    // ... and configure the backend NNS canisters:
    eprintln!("Configuring the NNS...");
    set_cmc_authorized_subnets(&nns_url, &default_subnet_id.to_string(), dfx_cache_path)?;

    print_nns_details(provider_url)?;
    Ok(())
}

/// Gets and checks the provider URL
///
/// # Errors
/// - The provider may be malformed.
/// - Only provider localhost:8080 is supported; this is compiled into the canister IDs.
///   - The port constraint may be eased, perhaps, at some stage.
///   - The requirement that the domain root is 'localhost' is less likely to change; 127.0.0.1 doesn't support subdomains.
#[context("Failed to get a valid provider for network '{}'.  Please check networks.json and dfx.json.", network_descriptor.name)]
fn get_and_check_provider(network_descriptor: &NetworkDescriptor) -> anyhow::Result<Url> {
    let provider_url = network_descriptor
        .first_provider()
        .with_context(|| "Environment has no providers")?;
    let provider_url: Url = Url::parse(provider_url)
        .with_context(|| "Malformed provider URL in this environment: {url_str}")?;

    if provider_url.port() != Some(8080) {
        return Err(anyhow!(
            "dfx nns install supports only port 8080, not {provider_url}. Please set the 'local' network's provider to '127.0.0.1:8080'."
        ));
    }

    Ok(provider_url)
}

/// Gets the local replica URL.  Note: This is not the same as the provider URL.
///
/// The replica URL hosts the canister dashboard and is used for installing NNS wasms.
///
/// Note: The port typically changes every time `dfx start --clean` is run.
///
/// # Errors
/// - Returns an error if the replica URL could not be found.  Typically this indicates that the local replica
///   is not running or is running in a different location.
/// - Returns an error if the network name is not "local"; that is the only network that `ic nns install` can deploy to.
///
/// # Panics
/// This code is not expected to panic.
#[context("Failed to determine the replica URL for network '{}'.  This may be caused by `dfx start` failing.",network_descriptor.name)]
pub fn get_and_check_replica_url(
    network_descriptor: &NetworkDescriptor,
    logger: &Logger,
) -> anyhow::Result<Url> {
    if network_descriptor.name != "local" {
        return Err(anyhow!(
            "dfx nns install can only deploy to the 'local' network."
        ));
    }
    network_descriptor
        .get_replica_urls(Some(logger))?
        .pop()
        .ok_or_else(|| {
            anyhow!("The list of replica URLs is empty; `dfx start` appears to be unhealthy.")
        })
}

/// Gets the subnet ID
#[context("Failed to determine subnet ID.")]
async fn get_subnet_id(agent: &Agent) -> anyhow::Result<Principal> {
    let root_key = agent
        .status()
        .await
        .with_context(|| "Could not get agent status")?
        .root_key
        .with_context(|| "Agent should have fetched the root key.")?;
    Ok(Principal::self_authenticating(root_key))
}

/// The NNS canisters use the very first few canister IDs; they must be available.
#[context("Failed to verify that the network is empty; dfx nns install must be run just after dfx start --clean")]
async fn verify_nns_canister_ids_are_available(agent: &Agent) -> anyhow::Result<()> {
    /// Checks that the canister is unused on the given network.
    ///
    /// The network is queried directly; local state such as canister_ids.json has no effect on this function.
    async fn verify_canister_id_is_available(
        agent: &Agent,
        canister_id: &str,
        canister_name: &str,
    ) -> anyhow::Result<()> {
        let canister_principal = Principal::from_text(canister_id).with_context(|| {
            format!("Internal error: {canister_name} has an invalid canister ID: {canister_id}")
        })?;
        if agent
            .read_state_canister_info(canister_principal, "module_hash")
            .await
            .is_ok()
        {
            return Err(anyhow!(
                "The ID for the {canister_name} canister has already been taken."
            ));
        }
        Ok(())
    }

    try_join_all(NNS_CORE.iter().cloned().map(
        |IcNnsInitCanister {
             canister_id,
             canister_name,
             ..
         }| verify_canister_id_is_available(agent, canister_id, canister_name),
    ))
    .await?;
    Ok(())
}

/// Provides the user with a printout detailing what has been installed for them.
///
/// # Errors
/// - May fail if the provider URL is invalid.
#[context("Failed to print NNS details.")]
fn print_nns_details(provider_url: Url) -> anyhow::Result<()> {
    let canister_url = |canister_id: &str| -> anyhow::Result<String> {
        let mut url = provider_url.clone();
        let host = format!("{}.localhost", canister_id);
        url.set_host(Some(&host))
            .with_context(|| "Could not add canister ID as a subdomain to localhost")?;
        Ok(url.to_string())
    };

    println!(
        r#"

######################################
# NNS CANISTER INSTALLATION COMPLETE #
######################################

Backend canisters:
{}

Frontend canisters:
{}

"#,
        NNS_CORE
            .iter()
            .map(|canister| format!("{:20}  {}\n", canister.canister_name, canister.canister_id))
            .collect::<Vec<String>>()
            .join(""),
        NNS_FRONTEND
            .iter()
            .map(|canister| format!(
                "{:20}  {}\n",
                canister.canister_name,
                canister_url(canister.canister_id).unwrap_or_default()
            ))
            .collect::<Vec<String>>()
            .join("")
    );
    Ok(())
}

/// Gets a URL, trying repeatedly until it is available.
#[context("Failed to download after multiple tries: {}", url)]
pub async fn get_with_retries(url: &Url) -> anyhow::Result<reqwest::Response> {
    let mut retry_policy = ExponentialBackoff::default();

    loop {
        match reqwest::get(url.clone()).await {
            Ok(response) => {
                return Ok(response);
            }
            Err(err) => match retry_policy.next_backoff() {
                Some(duration) => tokio::time::sleep(duration).await,
                None => bail!(err),
            },
        }
    }
}

/// Gets the local replica type from dfx.json
///
/// # Errors
/// Returns an error if the replica type could not be determined.  Possible reasons include:
/// - There is no `dfx.json`
/// - `dfx.json` could not be read.
/// - `dfx.json` is not valid JSON.
/// - The replica type is not defined for the `local` network.
///
/// # Panics
/// This code is not expected to panic.
#[context("Failed to determine the local replica type.")]
fn local_replica_type(network_descriptor: &NetworkDescriptor) -> anyhow::Result<ReplicaSubnetType> {
    Ok(network_descriptor
        .local_server_descriptor()?
        .replica
        .subnet_type
        .unwrap_or_default())
}

/// Checks that the local replica type is 'system'.
///
/// Note: At present dfx runs a single local replica and the replica type is taken from dfx.json.  It is unfortunate that the subnet type is forced
/// on the other canisters, however in practice this is unlikely to be a huge problem in the short term.
///
/// # Errors
/// - Returns an error if the local replica type in `dfx.json` is not "system".
/// # Panics
/// This code is not expected to panic.
#[context("Failed to verify that the local replica type is 'system'.")]
pub fn verify_local_replica_type_is_system(
    network_descriptor: &NetworkDescriptor,
    networks_config: &NetworksConfig,
) -> anyhow::Result<()> {
    match local_replica_type(network_descriptor) {
        Ok(ReplicaSubnetType::System) => Ok(()),
        other => Err(anyhow!(
            r#"The replica subnet_type needs to be 'system' to run NNS canisters. Current value: {other:?}.

             You can configure it by setting local.replica.subnet_type to "system" in your global networks.json:

             1) Create or edit: {}
             2) Set the local config to:
                 {{
                   "local": {{
                     "bind": "127.0.0.1:8080",
                     "type": "ephemeral",
                     "replica": {{
                       "subnet_type": "system"
                     }}
                   }}
                 }}
             3) Verify that you have no network configurations in dfx.json.
             4) Restart dfx:
                 dfx stop
                 dfx start --clean

             "#,
            networks_config.get_path().to_string_lossy()
        )),
    }
}

/// Downloads a file
#[context("Failed to download '{:?}' to '{:?}'.", source, target)]
pub async fn download(source: &Url, target: &Path) -> anyhow::Result<()> {
    if target.exists() {
        println!("Already downloaded: {}", target.to_string_lossy());
        return Ok(());
    }
    println!(
        "Downloading {}\n  from: {}",
        target.to_string_lossy(),
        source.as_str()
    );
    let buffer = reqwest::get(source.clone())
        .await
        .with_context(|| "Failed to connect")?
        .bytes()
        .await
        .with_context(|| "Download was interrupted")?;
    let target_parent = target
        .parent()
        .unwrap_or_else(|| Path::new(Component::CurDir.as_os_str()));
    let tmp_dir = tempfile::TempDir::new_in(target_parent)
        .with_context(|| "Failed to create temporary directory for download")?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join("wasm");
        let mut file = fs::File::create(&filename)
            .with_context(|| format!("Failed to create temp file at '{}'", filename.display()))?;
        file.write_all(&buffer)
            .with_context(|| format!("Failed to write temp file at '{}'.", filename.display()))?;
        filename
    };
    fs::rename(&downloaded_filename, target).with_context(|| {
        format!(
            "Failed to rename '{}' to '{}'",
            downloaded_filename.display(),
            target.display()
        )
    })?;
    Ok(())
}

/// Arguments for the ic-nns-init command line function.
pub struct IcNnsInitOpts {
    /// An URL to access one or more NNS subnet replicas.
    nns_url: String,
    /// A directory that needs to be populated will all required wasms before calling ic-nns-init.
    wasm_dir: PathBuf,
    /// The ID of a test account that ic-nns-init will create and to initialise with tokens.
    /// Note: At present only one test account is supported.
    test_accounts: Vec<String>,
    /// A subnet for SNS canisters.
    /// Note: In this context we support at most one subnet.
    sns_subnets: Option<String>,
    /// A file storing the registry content.
    local_registry_file: Option<PathBuf>,
}

/// Calls the `ic-nns-init` executable.
///
/// Notes:
///   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different binary for local development
///   - This won't work with an HSM, because the agent holds a session open
///   - The provider_url is what the agent connects to, and forwards to the replica.
#[context("Failed to install NNS components.")]
pub async fn ic_nns_init(opts: &IcNnsInitOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let mut args: Vec<OsString> = vec![
        "--pass-specified-id".into(),
        "--url".into(),
        opts.nns_url.clone().into(),
        "--wasm-dir".into(),
        opts.wasm_dir.as_os_str().into(),
    ];
    if let Some(local_registry_file) = &opts.local_registry_file {
        args.push("--initial-registry".into());
        args.push(local_registry_file.into());
    }
    for account in &opts.test_accounts {
        args.push("--initialize-ledger-with-test-accounts".into());
        args.push(account.into());
    }
    if let Some(subnets) = &opts.sns_subnets {
        args.push("--sns-subnet".into());
        args.push(subnets.into());
    }
    call_extension_bundled_binary("ic-nns-init", &args, dfx_cache_path)
}

/// Sets the subnets the CMC is authorized to create canisters in.
#[context("Failed to authorize a subnet for use by the cycles management canister. The CMC may not be able to create canisters.")]
pub fn set_cmc_authorized_subnets(
    nns_url: &Url,
    subnet: &str,
    dfx_cache_path: &Path,
) -> anyhow::Result<()> {
    let summary = format!(
        "Authorize the Cycles Minting Canister to create canisters in the subnet '{}'.",
        subnet
    );
    let args = vec![
        "--nns-url",
        nns_url.as_str(),
        "propose-to-set-authorized-subnetworks",
        "--test-neuron-proposer",
        "--proposal-title",
        "Set Cycles Minting Canister Authorized Subnets",
        "--summary",
        &summary,
        "--subnets",
        subnet,
    ];
    call_extension_bundled_binary("ic-admin", args, dfx_cache_path)
        .map_err(|e| anyhow!("Call to propose to set authorized subnets failed: {e}"))
}

/// Uploads wasms to the nns-sns-wasm canister.
#[context("Failed to upload wasm files to the nns-sns-wasm canister; it may not be possible to create an SNS.")]
pub fn upload_nns_sns_wasms_canister_wasms(dfx_cache_path: &Path) -> anyhow::Result<()> {
    for SnsCanisterInstallation {
        upload_name,
        wasm_name,
        ..
    } in SNS_CANISTERS
    {
        let wasm_path = nns_wasm_dir(dfx_cache_path).join(wasm_name);
        add_sns_wasm_for_tests(AddSnsWasmForTestsArgs {
            wasm_file: wasm_path.clone(),
            canister_type: upload_name.to_string(),
            override_sns_wasm_canister_id_for_tests: Some(NNS_SNS_WASM.canister_id.into()),
            network: "local".to_string(),
        })?;
    }
    Ok(())
}

/// Installs a canister without adding it to `dfx.json` or `canister_ids.json`.
///
/// # Errors
/// - Returns an error if the canister could not be created.
/// # Panics
/// None
//
// Notes:
// - This does not pass any initialisation argument.  If needed, one can be added to the code.
// - This function may be needed by other plugins as well.
#[context("Failed to install canister '{canister_name}' on network '{}' using wasm at '{}'.", network_descriptor.name, wasm_path.display())]
pub async fn install_canister(
    network_descriptor: &NetworkDescriptor,
    agent: &Agent,
    canister_name: &str,
    wasm_path: &Path,
    specified_id: Principal,
    init_arg: Option<&[u8]>,
) -> anyhow::Result<Principal> {
    let mgr = ManagementCanister::create(agent);
    let builder = mgr
        .create_canister()
        .as_provisional_create_with_specified_id(specified_id);

    let res = builder.call_and_wait().await;
    let canister_id: Principal = res.context("Canister creation call failed.")?.0;
    let canister_id_str = canister_id.to_text();

    let unit_args = Encode!(&())?;
    let install_args = init_arg.unwrap_or(&unit_args);
    let install_mode = InstallMode::Install;
    let call_sender = CallSender::SelectedId;
    fn ask_for_consent(_: &str) -> Result<(), UserConsent> {
        Ok(())
    }

    install_canister_wasm(
        agent,
        canister_id,
        Some(canister_name),
        install_args,
        install_mode,
        &call_sender,
        fs::read(wasm_path).with_context(|| format!("Unable to read {:?}", wasm_path))?,
        ask_for_consent,
    )
    .await?;

    println!("Installed {canister_name} at {canister_id_str}");

    Ok(canister_id)
}
