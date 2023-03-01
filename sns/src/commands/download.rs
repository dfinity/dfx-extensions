//! Code for the command line `dfx sns import`
use crate::lib::call_bundled::replica_rev;
use crate::lib::download::download_sns_wasms;
use std::path::PathBuf;

// use crate::lib::info::replica_rev;

use clap::Parser;
use tokio::runtime::Runtime;

/// Downloads the SNS canister WASMs
#[derive(Parser)]
pub struct SnsDownloadOpts {
    /// IC commit of SNS canister WASMs to download
    #[clap(long, env("DFX_IC_COMMIT"))]
    ic_commit: Option<String>,
    /// Path to store downloaded SNS canister WASMs
    #[clap(long, default_value = ".")]
    wasms_dir: PathBuf,
}

/// Executes the command line `dfx sns import`.
pub fn exec(opts: SnsDownloadOpts) -> anyhow::Result<()> {
    let runtime = Runtime::new().expect("Unable to create a runtime");
    let ic_commit = opts.ic_commit.unwrap_or_else(|| replica_rev().unwrap());
    runtime.block_on(download_sns_wasms(&ic_commit, &opts.wasms_dir))
}

// TODO
