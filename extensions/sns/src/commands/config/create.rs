//! Code for executing `dfx sns config create`
use crate::create_config::create_config;
use crate::CONFIG_FILE_NAME;
use clap::Parser;
use dfx_core::config::cache::get_bin_cache;
use dfx_core::config::model::dfinity::Config;
use dfx_extensions_utils::dfx_version;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

/// Executes `dfx sns config create`
pub fn exec(_opts: CreateOpts) -> anyhow::Result<()> {
    let sns_config_path = if let Some(config) = Config::from_current_dir()? {
        config.get_project_root().join(CONFIG_FILE_NAME)
    } else {
        anyhow::bail!("No config file found. Please run `dfx config create` first.");
    };
    let cache_path = get_bin_cache(&dfx_version()?)?;

    create_config(cache_path, &sns_config_path)?;
    println!(
        "Created SNS configuration at: {}",
        sns_config_path.display()
    );
    Ok(())
}
