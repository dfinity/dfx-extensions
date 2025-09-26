use crate::dependencies::execute_command;
use crate::error::dfx_executable::DfxError;
use fn_error_context::context;
use semver::Version;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

/// The replica revision of the NNS/SNS canisters which might have dependencies among each other.
/// It is highly recommended that this be kept in sync with the commit mentioned in the root Cargo.toml file.
pub const NNS_SNS_REPLICA_REV: &str = "721df73b943e87e2dad1d931819a2051401209a6";

/// Calls a binary from dfx cache.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
///
/// Does not print stdout/stderr to the console, and instead returns the output to the caller after the process has exited.
#[context("Calling {} CLI failed, or, it returned an error.", command)]
pub fn call_dfx_bundled_binary<S, I>(
    command: &str,
    args: I,
    dfx_cache_path: &Path,
) -> anyhow::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = dfx_cache_path.join(command);
    execute_command(&binary, args, dfx_cache_path)
}

pub fn dfx_version(dfx_cache_path: &Path) -> Result<String, DfxError> {
    let args = ["--version"];
    let version_cmd_output = Command::new(dfx_cache_path.join("dfx"))
        .args(args)
        .output()
        .map_err(DfxError::DfxExecutableError)?
        .stdout
        .iter()
        .map(|c| *c as char)
        .collect::<String>();
    if let Some(version) = version_cmd_output.split_whitespace().last() {
        Version::parse(version) // make sure the output is really a version
            .map_err(DfxError::DfxVersionMalformed)
            .map(|v| v.to_string())
    } else {
        Err(DfxError::MalformedCommandOutput {
            command: args.join(" ").to_string(),
            output: version_cmd_output,
        })
    }
}
