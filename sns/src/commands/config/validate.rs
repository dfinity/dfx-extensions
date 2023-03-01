//! Code for executing `dfx sns config validate`
use crate::lib;
use crate::lib::validate_config::validate_config;
use clap::Parser;
use dfx_core::config::model::dfinity::Config;

/// Validates an SNS configuration
#[derive(Parser)]
pub struct ValidateOpts {}

/// Executes `dfx sns config validate`
pub fn exec(_opts: ValidateOpts) -> anyhow::Result<()> {
    let config = Config::from_current_dir()?.unwrap();
    let path = config.get_project_root().join(lib::CONFIG_FILE_NAME);

    validate_config(&path).map(|stdout| println!("{}", stdout))
}
