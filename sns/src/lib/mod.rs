//! Code for decentralizing dapps
#![warn(clippy::missing_docs_in_private_items)]
pub mod call_bundled;
pub mod create_config;
pub mod deploy;
pub mod download;
pub mod project;
pub mod validate_config;

/// The default location of an SNS configuration file.
pub const CONFIG_FILE_NAME: &str = "sns.yml";
