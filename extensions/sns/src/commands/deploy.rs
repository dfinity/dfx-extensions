//! Code for the command line `dfx sns deploy`.
use crate::deploy::deploy_sns;
use crate::CONFIG_FILE_NAME;
use clap::Parser;
use dfx_core::config::cache::get_bin_cache;
use dfx_core::config::model::dfinity::Config;
use dfx_extensions_utils::dfx_version;

/// Creates an SNS on a network.
///
/// # Arguments
/// - `env` - The execution environment, including the network to deploy to and connection credentials.
/// - `opts` - Deployment options.
#[derive(Parser)]
pub struct DeployOpts {}

/// Executes the command line `dfx sns deploy`.
pub fn exec(_opts: DeployOpts) -> anyhow::Result<()> {
    println!("Creating SNS canisters.  This typically takes about one minute...");
    let sns_config_path = if let Some(config) = Config::from_current_dir()? {
        config.get_project_root().join(CONFIG_FILE_NAME)
    } else {
        anyhow::bail!("No config file found. Please run `dfx config create` first.");
    };
    let cache_path = get_bin_cache(&dfx_version()?)?;

    println!("{}", deploy_sns(cache_path, &sns_config_path)?);
    Ok(())
}
