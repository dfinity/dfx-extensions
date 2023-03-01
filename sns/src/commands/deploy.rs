//! Code for the command line `dfx sns deploy`.

use crate::lib;
use crate::lib::deploy::deploy_sns;
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
pub fn exec(_opts: DeployOpts) -> anyhow::Result<()> {
    println!("Creating SNS canisters.  This typically takes about one minute...");
    let config = Config::from_current_dir()?.unwrap();
    let path = config.get_project_root().join(lib::CONFIG_FILE_NAME);

    println!("{}", deploy_sns(&path)?);
    Ok(())
}
