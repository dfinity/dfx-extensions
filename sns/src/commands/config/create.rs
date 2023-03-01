//! Code for executing `dfx sns config create`

use crate::lib;
use crate::lib::create_config::create_config;
use clap::Parser;
use dfx_core::config::model::dfinity::Config;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

/// Executes `dfx sns config create`
pub fn exec(_opts: CreateOpts) -> anyhow::Result<()> {
    let config = Config::from_current_dir()?.unwrap();
    let path = config.get_project_root().join(lib::CONFIG_FILE_NAME);

    create_config(&path)?;
    println!("Created SNS configuration at: {}", path.display());
    Ok(())
}
