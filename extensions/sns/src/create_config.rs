//! Code for creating SNS configurations
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use dfx_extensions_utils::call_bundled;

/// Ceates an SNS configuration template.
#[context("Failed to create sns config at {}.", path.display())]
pub fn create_config(dfx_cache_path: &Path, path: &Path) -> anyhow::Result<()> {
    let args = vec![
        OsString::from("init-config-file"),
        OsString::from("--init-config-file-path"),
        OsString::from(path),
        OsString::from("new"),
    ];
    call_bundled(&dfx_cache_path, "sns", &args)?;
    Ok(())
}
