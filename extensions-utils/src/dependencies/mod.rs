use anyhow::anyhow;
use std::{
    env,
    ffi::OsStr,
    path::Path,
    process::{self, Command},
};

pub mod call;
pub mod dfx;
pub mod download_ic_binaries;
pub mod download_wasms;

pub fn execute_command(
    binary_path: &Path,
    args: impl IntoIterator<Item = impl AsRef<OsStr>>,
    dfx_cache_path: &Path,
) -> anyhow::Result<String> {
    let mut command = Command::new(binary_path);
    command.args(args);
    if let Some(old_path) = env::var_os("PATH") {
        let mut paths = env::split_paths(&old_path).collect::<Vec<_>>();
        paths.push(dfx_cache_path.to_path_buf());
        let new_path = env::join_paths(paths)?;
        command.env("PATH", new_path);
    } else {
        command.env("PATH", dfx_cache_path);
    }
    command.stdin(process::Stdio::null());
    command.output().map_err(anyhow::Error::from).and_then(
        |output| -> Result<String, anyhow::Error> {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                Err(anyhow!(
                    "Command failed:\n{:?}\nStdout:\n{}\n\nStderr:\n{}",
                    command,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        },
    )
}
