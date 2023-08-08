//! Code for executing `dfx sns prepare-canisters add-nns-root`
use candid::Principal;
use clap::Parser;
use dfx_extensions_utils::call_extension_bundled_binary;
use std::ffi::OsString;

/// `dfx sns prepare-canisters add-nns-root` command line arguments.
#[derive(Parser)]
pub struct AddNnsRootOpts {
    // TODO support multiple canisters?
    canister_id: Principal,
}

/// Executes `dfx sns prepare-canisters add-nns-root`
pub fn exec(opts: AddNnsRootOpts) -> anyhow::Result<()> {
    let args = vec![
        OsString::from("prepare-canisters"),
        OsString::from("add-nns-root"),
        OsString::from(opts.canister_id.to_string()),
    ];

    call_extension_bundled_binary("sns-cli", &args)?;
    Ok(())
}
