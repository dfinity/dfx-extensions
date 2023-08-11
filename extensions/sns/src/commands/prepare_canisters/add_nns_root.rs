//! Code for executing `dfx sns prepare-canisters add-nns-root`
use candid::Principal;
use clap::Parser;
use dfx_extensions_utils::call_extension_bundled_binary;
use std::ffi::OsString;
use std::path::Path;

/// `dfx sns prepare-canisters add-nns-root` command line arguments.
#[derive(Parser)]
pub struct AddNnsRootOpts {
    canister_ids: Vec<Principal>,
}

/// Executes `dfx sns prepare-canisters add-nns-root`
pub fn exec(opts: AddNnsRootOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    let mut args = vec![
        OsString::from("prepare-canisters"),
        OsString::from("add-nns-root"),
    ];
    args.extend(opts.canister_ids.iter().map(|id| id.to_string().into()));

    call_extension_bundled_binary("sns-cli", &args, dfx_cache_path)
        .map(|stdout| println!("{}", stdout))
        .map_err(|error| {
            println!("{}", error);
            error
        })?;
    Ok(())
}
