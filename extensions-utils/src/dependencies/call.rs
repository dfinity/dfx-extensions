use crate::dependencies::execute_command;
use anyhow::anyhow;
use fn_error_context::context;
use std::{env, ffi::OsStr, path::Path};

/// Calls a binary that was delivered with an extension tarball.
///
/// # Returns
/// - On success, returns stdout as a string.
/// - On error, returns an error message including stdout and stderr.
///
/// Does not print stdout/stderr to the console, and instead returns the output to the caller after the process has exited.
#[context("Calling {} CLI failed, or, it returned an error.", binary_name)]
pub fn call_extension_bundled_binary<S, I>(
    binary_name: &str,
    args: I,
    dfx_cache_path: &Path,
) -> anyhow::Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let extension_binary_path =
        env::current_exe().map_err(|e| anyhow!("Failed to get current exe: {}", e))?;
    let extension_dir_path = extension_binary_path.parent().ok_or_else(|| {
        anyhow!(
            "Failed to locate parent of dir of executable: {}",
            extension_binary_path.display()
        )
    })?;
    let binary_to_call = extension_dir_path.join(binary_name);
    execute_command(&binary_to_call, args, dfx_cache_path)
}
