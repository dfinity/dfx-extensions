//! Code for the command line `dfx sns import`
use dfx_extensions_utils::download_sns_wasms;
use dfx_extensions_utils::replica_rev;

use clap::Parser;
use std::path::{Path, PathBuf};
use tokio::runtime::Runtime;

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
pub fn exec(opts: SnsDownloadOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_commit = if let Some(ic_commit) = opts.ic_commit {
        ic_commit
    } else {
        replica_rev(dfx_cache_path)?
    };
    runtime.block_on(download_sns_wasms(&ic_commit, &opts.wasms_dir))?;
    Ok(())
}
