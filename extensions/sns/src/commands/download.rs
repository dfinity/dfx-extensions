//! Code for the command line `dfx sns import`
use clap::Parser;
use dfx_extensions_utils::{dependencies::dfx::NNS_SNS_REPLICA_REV, download_sns_wasms};
use std::path::{Path, PathBuf};

/// Downloads the SNS canister WASMs
#[derive(Parser)]
pub struct SnsDownloadOpts {
    /// IC commit of SNS canister WASMs to download
    #[arg(long, env = "DFX_IC_COMMIT")]
    ic_commit: Option<String>,
    /// Path to store downloaded SNS canister WASMs
    #[arg(long, default_value = ".")]
    wasms_dir: PathBuf,
}

/// Executes the command line `dfx sns import`.
pub async fn exec(opts: SnsDownloadOpts) -> anyhow::Result<()> {
    let ic_commit = opts.ic_commit.unwrap_or(NNS_SNS_REPLICA_REV.to_string());
    download_sns_wasms(&ic_commit, &opts.wasms_dir).await?;
    Ok(())
}
