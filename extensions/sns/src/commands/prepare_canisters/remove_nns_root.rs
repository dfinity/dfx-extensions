//! Code for executing `dfx sns prepare-canisters remove-nns-root`
use std::ffi::OsString;
use std::path::Path;
use clap::Parser;
use candid::Principal;
use dfx_extensions_utils::call_bundled;

/// `dfx sns prepare-canisters remove-nns-root` command line arguments.
#[derive(Parser)]
pub struct RemoveNnsRootOpts {
    // TODO support multiple canisters?
    canister_id: Principal,
}

/// Executes `dfx sns prepare-canisters remove-nns-root`
pub fn exec(opts: RemoveNnsRootOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {

    let args = vec![
        OsString::from("prepare-canisters"),
        OsString::from("remove-nns-root"),
        OsString::from(opts.canister_id.to_string()),
    ];

    call_bundled(&dfx_cache_path, "sns", &args)?;
    Ok(())
}