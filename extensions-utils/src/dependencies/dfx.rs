use crate::dependencies::execute_command;
use crate::error::dfx_executable::DfxError;
use fn_error_context::context;
use semver::Version;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

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
) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = dfx_cache_path.join(command);
    execute_command(&binary, args, dfx_cache_path)
}

pub fn replica_rev(dfx_cache_path: &Path) -> Result<String, DfxError> {
    let args = ["info", "replica-rev"];
    let rev = Command::new(dfx_cache_path.join("dfx"))
        .args(args)
        .output()
        .map_err(DfxError::DfxExecutableError)?
        .stdout
        .iter()
        .map(|c| *c as char)
        .collect::<String>()
        .trim()
        .to_string();
    if rev.len() != 40 {
        return Err(DfxError::MalformedCommandOutput {
            command: args.join(" ").to_string(),
            output: rev,
        });
    }
    Ok(rev)
}

pub fn webserver_port(dfx_cache_path: &Path) -> Result<u16, DfxError> {
    let args = ["info", "webserver-port"];
    let output = Command::new(dfx_cache_path.join("dfx"))
        .args(args)
        .output()
        .map_err(DfxError::DfxExecutableError)?
        .stdout
        .iter()
        .map(|c| *c as char)
        .collect::<String>();
    let port = output.trim().parse::<u16>();
    if port.is_err() {
        return Err(DfxError::MalformedCommandOutput {
            command: args.join(" ").to_string(),
            output: output.to_string(),
        });
    }
    Ok(port.unwrap())
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
