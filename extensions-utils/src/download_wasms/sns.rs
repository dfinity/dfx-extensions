use std::path::Path;

use anyhow;
use fn_error_context::context;
use futures_util::future::try_join_all;

use crate::download_ic_repo_wasm;

/// Downloads all the core SNS wasms.
#[context("Failed to download SNS wasm files.")]
pub async fn download_sns_wasms(ic_commit: &str, wasms_dir: &Path) -> anyhow::Result<()> {
    try_join_all(
        SNS_CANISTERS
            .iter()
            .map(|SnsCanisterInstallation { wasm_name, .. }| {
                download_ic_repo_wasm(wasm_name, ic_commit, wasms_dir)
            }),
    )
    .await?;
    Ok(())
}

/// Information required for WASMs uploaded to the nns-sns-wasm canister.
///
/// Note:  These wasms are not deployed by `ic nns install` but later by developers
pub struct SnsCanisterInstallation {
    /// The name of the canister as typically added to dfx.json or used in `dfx cansiter id NAME`
    pub canister_name: &'static str,
    /// The name used when uploading to the nns-sns-wasm canister.
    pub upload_name: &'static str,
    /// The basename of the wasm file.
    pub wasm_name: &'static str,
}
/// The controller of all the canisters in a given SNS project.
pub const SNS_ROOT: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-root",
    upload_name: "root",
    wasm_name: "sns-root-canister.wasm",
};
/// The governance canister for an SNS project.
pub const SNS_GOVERNANCE: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-governance",
    upload_name: "governance",
    wasm_name: "sns-governance-canister.wasm",
};
/// Manages the decentralisation of an SNS project, exchanging stake in the mainnet for stake in the project.
pub const SNS_SWAP: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-swap",
    upload_name: "swap",
    wasm_name: "sns-swap-canister.wasm",
};
/// Stores account balances for an SNS project.
pub const SNS_LEDGER: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-ledger",
    upload_name: "ledger",
    wasm_name: "ic-icrc1-ledger.wasm",
};
/// Stores ledger data; needed to preserve data if an SNS ledger canister is upgraded.
pub const SNS_LEDGER_ARCHIVE: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-ledger-archive",
    upload_name: "archive",
    wasm_name: "ic-icrc1-archive.wasm",
};
/// Indexes ledger data.
pub const SNS_INDEX: SnsCanisterInstallation = SnsCanisterInstallation {
    canister_name: "sns-index",
    upload_name: "index",
    wasm_name: "ic-icrc1-index.wasm",
};
/// SNS wasm files hosted by the nns-sns-wasms canister.
///
/// Note:  Sets of these canisters are deployed on request, so one network will
/// typically have many sets of these canisters, one per project decentralized
/// with the SNS toolchain.
pub const SNS_CANISTERS: [&SnsCanisterInstallation; 6] = [
    &SNS_ROOT,
    &SNS_GOVERNANCE,
    &SNS_SWAP,
    &SNS_LEDGER,
    &SNS_LEDGER_ARCHIVE,
    &SNS_INDEX,
];
