//! Code for checking SNS config file validity
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use dfx_extensions_utils::call_extension_bundled_binary;

/// Checks whether an SNS configuration file is valid.
#[context("Failed to validate SNS config at {}.", path.display())]
pub fn validate_config(path: &Path) -> anyhow::Result<String> {
    let args = vec![
        OsString::from("init-config-file"),
        OsString::from("--init-config-file-path"),
        OsString::from(path),
        OsString::from("validate"),
    ];
    // current binary

    call_extension_bundled_binary("sns-cli", &args)
        .map(|_| format!("SNS config file is valid: {}", path.display()))
}
