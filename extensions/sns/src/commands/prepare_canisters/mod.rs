//! Code for command line `dfx sns prepare-canisters
use crate::commands::prepare_canisters::{
    add_nns_root::AddNnsRootOpts, remove_nns_root::RemoveNnsRootOpts,
};
use clap::Parser;
use std::path::Path;

mod add_nns_root;
mod remove_nns_root;

/// SNS prepare-canisters command line arguments.
#[derive(Parser)]
#[command(name("prepare-canisters"))]
pub struct SnsPrepareCanistersOpts {
    /// `dfx sns prepare-canisters` subcommand arguments.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Command line options for `dfx sns prepare-canisters` subcommands.
#[derive(Parser)]
enum SubCommand {
    /// Command line options for adding NNS Root as a co-controller of a dapp canister.
    AddNnsRoot(AddNnsRootOpts),
    /// Command line options for removing NNS Root as a co-controller of a dapp canister.
    RemoveNnsRoot(RemoveNnsRootOpts),
}

/// Executes `dfx sns prepare-canisters` and its subcommands.
pub fn exec(opts: SnsPrepareCanistersOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    match opts.subcmd {
        SubCommand::AddNnsRoot(v) => add_nns_root::exec(v, dfx_cache_path),
        SubCommand::RemoveNnsRoot(v) => remove_nns_root::exec(v, dfx_cache_path),
    }
}
