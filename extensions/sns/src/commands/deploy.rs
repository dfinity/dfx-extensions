//! Code for the command line `dfx sns deploy`.
use std::path::Path;

use crate::deploy::deploy_sns;
use crate::CONFIG_FILE_NAME;
use clap::Parser;
use dfx_core::config::model::dfinity::Config;

/// Creates an SNS on a network.
///
/// # Arguments
/// - `env` - The execution environment, including the network to deploy to and connection credentials.
/// - `opts` - Deployment options.
#[derive(Parser)]
pub struct DeployOpts {}

/// Executes the command line `dfx sns deploy`.
pub fn exec(_opts: DeployOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    println!("Creating SNS canisters.  This typically takes about one minute...");
    let sns_config_path = if let Some(config) = Config::from_current_dir()? {
        config.get_project_root().join(CONFIG_FILE_NAME)
    } else {
        anyhow::bail!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?");
    };

    println!("{}", deploy_sns(dfx_cache_path, &sns_config_path)?);
    Ok(())
}
