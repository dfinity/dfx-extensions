//! Code for executing `dfx sns prepare-canisters remove-nns-root`
use candid::Principal;
use clap::Parser;
use dfx_extensions_utils::call_extension_bundled_binary;
use std::ffi::OsString;
use std::path::Path;

/// `dfx sns prepare-canisters remove-nns-root` command line arguments.
#[derive(Parser)]
pub struct RemoveNnsRootOpts {
    canister_ids: Vec<Principal>,
}

/// Executes `dfx sns prepare-canisters remove-nns-root`
pub fn exec(opts: RemoveNnsRootOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let mut args = vec![
        OsString::from("prepare-canisters"),
        OsString::from("remove-nns-root"),
    ];
    args.extend(opts.canister_ids.iter().map(|id| id.to_string().into()));

    call_extension_bundled_binary("sns-cli", &args, dfx_cache_path)?;
    Ok(())
}
