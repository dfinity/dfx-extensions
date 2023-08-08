use crate::error::dfx_executable::DfxError;
use anyhow::Context;
use fn_error_context::context;
use semver::Version;

use std::env;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{self, Command};

/// Calls a binary from dfx cache.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
///
/// Does not stream stdout/stderr, and instead returns it after the process has exited.
#[context("Calling {} CLI, or, it returned an error.", command)]
pub fn call_dfx_bundled_binary<S, I>(
    dfx_cache_path: &Path,
    command: &str,
    args: I,
) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = dfx_cache_path.join(command);
    let mut command = Command::new(binary);
    // If extension's dependency calls dfx; it should call dfx in this dir.
    if let Some(path) = env::var_os("PATH") {
        let mut paths = env::split_paths(&path).collect::<Vec<_>>();
        paths.push(dfx_cache_path.to_path_buf());
        let new_path = env::join_paths(paths)?;
        command.env("PATH", new_path);
    } else {
        command.env("PATH", dfx_cache_path);
    }
    command.args(args);
    let output = command
        .stdin(process::Stdio::null())
        .output()
        .with_context(|| format!("Error executing {:#?}", command))?
        .stdout;
    Ok(String::from_utf8_lossy(&output).to_string())
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
