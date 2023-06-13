//! Code for the command line `dfx sns config`.
use std::path::Path;

use clap::Parser;

mod create;
mod validate;

/// Command line options for `sdx sns config`.
#[derive(Parser)]
pub struct ConfigOpts {}

/// SNS config command line arguments.
#[derive(Parser)]
#[command(name("config"))]
pub struct SnsConfigOpts {
    /// `dfx sns config` subcommand arguments.
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// Command line options for `sdx sns` subcommands.
#[derive(Parser)]
enum SubCommand {
    /// Command line options for creating an SNS configuration.
    Create(create::CreateOpts),
    /// Command line options for validating an SNS configuration.
    Validate(validate::ValidateOpts),
}

/// Executes `dfx sns config` and its subcommands.
pub fn exec(opts: SnsConfigOpts, dfx_cache_path: &Path) -> anyhow::Result<()> {
    match opts.subcmd {
        SubCommand::Create(v) => create::exec(v, dfx_cache_path),
        SubCommand::Validate(v) => validate::exec(v, dfx_cache_path),
    }
}
