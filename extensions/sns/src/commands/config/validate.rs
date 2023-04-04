//! Code for executing `dfx sns config validate`
use crate::validate_config::validate_config;
use crate::CONFIG_FILE_NAME;
use clap::Parser;
use dfx_core::config::cache::get_bin_cache;
use dfx_core::config::model::dfinity::Config;
use dfx_extensions_utils::dfx_version;

/// Validates an SNS configuration
#[derive(Parser)]
pub struct ValidateOpts {}

/// Executes `dfx sns config validate`
pub fn exec(_opts: ValidateOpts) -> anyhow::Result<()> {
    let sns_config_path = if let Some(config) = Config::from_current_dir()? {
        config.get_project_root().join(CONFIG_FILE_NAME)
    } else {
        anyhow::bail!("No config file found. Please run `dfx config create` first.");
    };
    let cache_path = get_bin_cache(&dfx_version()?)?;

    validate_config(cache_path, &sns_config_path).map(|stdout| println!("{}", stdout))
}
