//! Code for executing `dfx sns config create`
use crate::create_config::create_config;
use crate::CONFIG_FILE_NAME;
use clap::Parser;
use dfx_core::config::model::dfinity::Config;

/// Create an sns config
#[derive(Parser)]
pub struct CreateOpts {}

/// Executes `dfx sns config create`
pub fn exec(_opts: CreateOpts) -> anyhow::Result<()> {
    let sns_config_path = if let Some(config) = Config::from_current_dir()? {
        config.get_project_root().join(CONFIG_FILE_NAME)
    } else {
        anyhow::bail!(crate::errors::DFXJSON_NOT_FOUND);
    };

    create_config(&sns_config_path)?;
    println!(
        "Created SNS configuration at: {}",
        sns_config_path.display()
    );
    Ok(())
}
