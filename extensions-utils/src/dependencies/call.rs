use anyhow::Context;
use fn_error_context::context;
use std::{
    env,
    ffi::OsStr,
    path::Path,
    process::{self, Command},
};

/// Calls a binary that was delivered with an extension tarball.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
///
/// Does not stream stdout/stderr, and instead returns it after the process has exited.
#[context("Calling {} CLI failed, or, it returned an error.", binary_name)]
pub fn call_extension_bundled_binary<S, I>(
    dfx_cache_path: &Path,
    binary_name: &str,
    args: I,
) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let extension_binary_path =
        std::env::current_exe().map_err(|e| anyhow::anyhow!("Failed to get current exe: {}", e))?;
    let extension_dir_path = extension_binary_path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "Failed to locate parent of dir of executable: {}",
            extension_binary_path.display()
        )
    })?;
    let binary_to_call = extension_dir_path.join(binary_name);
    let mut command = Command::new(binary_to_call);
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
