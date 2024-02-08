//! Implements the `dfx nns install` command, which installs the core NNS canisters, including II and NNS-dapp.
//!
//! Note: `dfx nns` will be a `dfx` plugin, so this code SHOULD NOT depend on NNS code except where extremely inconvenient or absolutely necessary:
//! * Example: Minimise crate dependencies outside the nns modules.
//! * Example: Use `anyhow::Result` not `DfxResult`
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use dfx_core::canister::install_canister_wasm;
use dfx_core::config::model::dfinity::{NetworksConfig, ReplicaSubnetType};
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::identity::CallSender;
use dfx_extensions_utils::dependencies::download_wasms::nns::{CYCLES_LEDGER, NNS_CYCLES_MINTING};
use dfx_extensions_utils::{
    call_extension_bundled_binary, download_nns_wasms, nns_wasm_dir, IcNnsInitCanister,
    SnsCanisterInstallation, StandardCanister, ED25519_TEST_ACCOUNT, NNS_CORE, NNS_FRONTEND,
    NNS_SNS_WASM, SECP256K1_TEST_ACCOUNT, SNS_CANISTERS,
};

use anyhow::{anyhow, bail, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::{CandidType, Encode};
use fn_error_context::context;
use futures_util::future::try_join_all;
use ic_agent::export::Principal;
use ic_agent::Agent;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;
use reqwest::Url;
use sha2::{Digest, Sha256};
use slog::Logger;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::Component;
use std::path::{Path, PathBuf};

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
    logger: &Logger,
) -> anyhow::Result<()> {
    eprintln!("Checking out the environment...");
    verify_local_replica_type_is_system(network, networks_config)?;
    verify_nns_canister_ids_are_available(agent).await?;
    let provider_url = get_and_check_provider(network)?;
    let nns_url = get_and_check_replica_url(network, logger)?;
    let subnet_id = get_subnet_id(agent).await?.to_text();

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
        sns_subnets: Some(subnet_id.to_string()),
    };
    ic_nns_init(&ic_nns_init_opts, dfx_cache_path).await?;

    eprintln!("Uploading NNS configuration data...");
    upload_nns_sns_wasms_canister_wasms(dfx_cache_path)?;

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
        let installed_canister_id = install_canister(
            network,
            agent,
            canister_name,
            &local_wasm_path,
            specified_id,
            logger,
        )
        .await?
        .to_text();
        if canister_id != &installed_canister_id {
            bail!("Canister '{canister_name}' was installed at an incorrect canister ID.  Expected '{canister_id}' but got '{installed_canister_id}'.");
        }
    }
    // ... and configure the backend NNS canisters:
    eprintln!("Configuring the NNS...");
    set_xdr_rate(1234567, &nns_url, dfx_cache_path)?;
    set_cmc_authorized_subnets(&nns_url, &subnet_id, dfx_cache_path)?;
    set_cycles_ledger_canister_id_in_cmc(&nns_url, dfx_cache_path)?;

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

/// Sets the exchange rate between ICP and cycles.
///
/// # Implementation
/// This is done by proposal.  Just after starting a test server, ic-admin
/// proposals with a test user pass immediately, as the small test neuron is
/// the only neuron and has absolute majority.
#[context("Failed to set an initial exchange rate between ICP and cycles. It may not be possible to create canisters or purchase cycles.")]
pub fn set_xdr_rate(rate: u64, nns_url: &Url, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let summary = format!("Set the cycle exchange rate to {}.", rate.clone());
    let xdr_permyriad_per_icp = rate.to_string();
    let args = vec![
        "--nns-url",
        nns_url.as_str(),
        "propose-xdr-icp-conversion-rate",
        "--test-neuron-proposer",
        "--summary",
        &summary,
        "--xdr-permyriad-per-icp",
        &xdr_permyriad_per_icp,
    ];
    call_extension_bundled_binary("ic-admin", args, dfx_cache_path)
        .map_err(|e| anyhow!("Call to propose to set xdr rate failed: {e}"))
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

/// Sets the cycles ledger canister id in the CMC
#[context("Failed to set the cycles ledger canister id in the CMC")]
pub fn set_cycles_ledger_canister_id_in_cmc(
    nns_url: &Url,
    dfx_cache_path: &Path,
) -> anyhow::Result<()> {
    #[derive(CandidType, Clone, Debug, PartialEq, Eq)]
    struct CyclesCanisterInitPayload {
        pub cycles_ledger_canister_id: Option<Principal>,
    }

    let wasm_path = nns_wasm_dir(dfx_cache_path);
    let cmc_wasm_path = wasm_path.join(NNS_CYCLES_MINTING.wasm_name);
    let cmc_wasm_bytes = dfx_core::fs::read(&cmc_wasm_path)?;
    let wasm_hash = Sha256::digest(cmc_wasm_bytes);
    let upgrade_arg = format!(
        "(opt record {{ cycles_ledger_canister_id = opt principal \"{}\" }})",
        CYCLES_LEDGER.canister_id
    );
    let mut upgrade_arg_file = tempfile::NamedTempFile::new()?;
    upgrade_arg_file
        .write_all(upgrade_arg.as_bytes())
        .context("Failed to write to tempfile.")?;

    let cmc_wasm_path_str = cmc_wasm_path.to_string_lossy();
    let wasm_hash_str = hex::encode(wasm_hash);
    let upgrade_arg_file_str = upgrade_arg_file.path().to_string_lossy();
    let args = vec![
        "--nns-url",
        nns_url.as_str(),
        "propose-to-change-nns-canister",
        "--test-neuron-proposer",
        "--proposal-title",
        "Set cycles ledger canister id in Cycles Minting Canister",
        "--summary",
        "Set cycles ledger canister id in Cycles Minting Canister",
        "--mode",
        "upgrade",
        "--canister-id",
        NNS_CYCLES_MINTING.canister_id,
        "--wasm-module-path",
        &cmc_wasm_path_str,
        "--wasm-module-sha256",
        &wasm_hash_str,
        "--arg",
        &upgrade_arg_file_str,
    ];
    call_extension_bundled_binary("ic-admin", args, dfx_cache_path)
        .map_err(|e| anyhow!("Call to set the cycles ledger canister id in the CMC: {e}"))
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
        let args = vec![
            "add-sns-wasm-for-tests".into(),
            "--network".into(),
            "local".into(),
            "--override-sns-wasm-canister-id-for-tests".into(),
            NNS_SNS_WASM.canister_id.into(),
            "--wasm-file".into(),
            wasm_path.clone().into_os_string(),
            upload_name.into(),
        ];
        call_extension_bundled_binary("sns-cli", &args, dfx_cache_path)
            .map_err(|e| anyhow!(
                        "Failed to upload {upload_name} from {wasm_path:?} to the nns-sns-wasm canister by calling `sns-cli`: {e}"
                    ))?;
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
    logger: &Logger,
) -> anyhow::Result<Principal> {
    let mgr = ManagementCanister::create(agent);
    let builder = mgr
        .create_canister()
        .as_provisional_create_with_specified_id(specified_id);

    let res = builder.call_and_wait().await;
    let canister_id: Principal = res.context("Canister creation call failed.")?.0;
    let canister_id_str = canister_id.to_text();

    let install_args = Encode!(&())?;
    let install_mode = InstallMode::Install;
    let call_sender = CallSender::SelectedId;

    install_canister_wasm(
        agent,
        canister_id,
        Some(canister_name),
        &install_args,
        install_mode,
        &call_sender,
        fs::read(wasm_path).with_context(|| format!("Unable to read {:?}", wasm_path))?,
        true,
        logger,
    )
    .await?;

    println!("Installed {canister_name} at {canister_id_str}");

    Ok(canister_id)
}
