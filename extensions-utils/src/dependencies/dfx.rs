use crate::error::dfx_executable::DfxError;
use anyhow::anyhow;
use fn_error_context::context;
use semver::Version;

use std::ffi::OsStr;
use std::path::Path;
use std::process::{self, Command};

/// Calls a binary from dfx cache.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
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
    let mut command = Command::new(&binary);
    // If extension's dependency calls dfx; it should call dfx in this dir.
    command.env("PATH", dfx_cache_path.join("dfx"));
    command.args(args);
    command
        .stdin(process::Stdio::null())
        .output()
        .map_err(anyhow::Error::from)
        .and_then(|output| {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                let args: Vec<_> = command
                    .get_args()
                    .into_iter()
                    .map(OsStr::to_string_lossy)
                    .collect();
                Err(anyhow!(
                    "Call failed:\n{:?} {}\nStdout:\n{}\n\nStderr:\n{}",
                    command.get_program(),
                    args.join(" "),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        })
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
        Version::parse(&version) // make sure the output is really a version
            .map_err(DfxError::DfxVersionMalformed)
            .map(|v| v.to_string())
    } else {
        Err(DfxError::MalformedCommandOutput {
            command: args.join(" ").to_string(),
            output: version_cmd_output,
        })
    }
}
