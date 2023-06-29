//! Code for executing `dfx sns config validate`
use std::path::Path;

use crate::validate_config::validate_config;
use crate::CONFIG_FILE_NAME;
use clap::Parser;
use dfx_core::config::model::dfinity::Config;

/// Validates an SNS configuration
#[derive(Parser)]
pub struct ValidateOpts {}

/// Executes `dfx sns config validate`
pub fn exec(_opts: ValidateOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let sns_config_path = if let Some(config) = Config::from_current_dir()? {
        config.get_project_root().join(CONFIG_FILE_NAME)
    } else {
        anyhow::bail!("Cannot find dfx configuration file in the current working directory. Did you forget to create one?");
    };
    validate_config(dfx_cache_path, &sns_config_path).map(|stdout| println!("{}", stdout))
}
