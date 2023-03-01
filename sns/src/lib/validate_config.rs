//! Code for checking SNS config file validity
use fn_error_context::context;
use std::ffi::OsString;
use std::path::Path;

use crate::lib::call_bundled::call_bundled;

/// Checks whether an SNS configuration file is valid.
#[context("Failed to validate SNS config at {}.", path.display())]
pub fn validate_config(path: &Path) -> anyhow::Result<String> {
    let args = vec![
        OsString::from("init-config-file"),
        OsString::from("--init-config-file-path"),
        OsString::from(path),
        OsString::from("validate"),
    ];
    call_bundled("sns", &args).map(|_| format!("SNS config file is valid: {}", path.display()))
}
