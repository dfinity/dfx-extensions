use crate::error::dfx_executable::DfxError;
use anyhow::anyhow;
use dfx_core::config::cache::{
    binary_command_from_version, delete_version, get_binary_path_from_version, is_version_installed,
};
use dfx_core::error::cache::CacheError;
use fn_error_context::context;
use semver::Version;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

/// Calls a bundled command line tool.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
#[context("Calling {} CLI, or, it returned an error.", command)]
pub fn call_bundled<S, I>(cache_path: PathBuf, command: &str, args: I) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let binary = cache_path.join(command);

    let mut command = Command::new(&binary);
    command.args(args);
    // The sns command line tool itself calls dfx; it should call this dfx.
    // The sns command line tool should not rely on commands not packaged with dfx.
    // The same applies to other bundled binaries.
    command.env("PATH", binary.parent().unwrap_or_else(|| Path::new(".")));
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

pub fn replica_rev() -> Result<String, DfxError> {
    let args = ["info", "replica-rev"];
    let rev = Command::new("dfx")
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

pub fn webserver_port() -> Result<u16, DfxError> {
    let args = ["info", "webserver-port"];
    let output = Command::new("dfx")
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

pub fn dfx_version() -> Result<String, DfxError> {
    let args = ["--version"];
    let version_cmd_output = Command::new("dfx")
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

pub struct Cache {
    dfx_verison: String,
}

impl Cache {
    pub fn from_version(version: &str) -> Result<Self, DfxError> {
        let dfx_verison = version.to_string();
        let cache = Self { dfx_verison };
        if !is_version_installed(&version).map_err(DfxError::DfxCacheError)? {
            return Err(DfxError::DfxCacheNotInstalled(version.to_string()));
        }
        Ok(cache)
    }
}

impl dfx_core::config::cache::Cache for Cache {
    fn version_str(&self) -> String {
        self.dfx_verison.clone()
    }

    fn is_installed(&self) -> Result<bool, CacheError> {
        is_version_installed(&self.version_str())
    }

    fn delete(&self) -> Result<(), CacheError> {
        delete_version(&self.version_str()).map(|_| {})
    }

    fn get_binary_command_path(&self, binary_name: &str) -> Result<PathBuf, CacheError> {
        get_binary_path_from_version(&self.version_str(), binary_name)
    }

    fn get_binary_command(&self, binary_name: &str) -> Result<std::process::Command, CacheError> {
        binary_command_from_version(&self.version_str(), binary_name)
    }
}
