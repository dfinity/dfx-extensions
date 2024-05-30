//! Library for calling bundled command line tools.

pub mod dependencies;
mod error;
mod logger;
mod project;

pub use dependencies::{
    call::call_extension_bundled_binary,
    dfx::{call_dfx_bundled_binary, dfx_version, replica_rev},
    download_ic_binaries::download_ic_binary,
    download_wasms::{
        download_ic_repo_wasm,
        nns::{
            download_nns_wasms, nns_wasm_dir, IcNnsInitCanister, StandardCanister,
            ED25519_TEST_ACCOUNT, NNS_CORE, NNS_CORE_MANUAL, NNS_FRONTEND, NNS_SNS_WASM,
            SECP256K1_TEST_ACCOUNT,
        },
        sns::{download_sns_wasms, SnsCanisterInstallation, SNS_CANISTERS},
    },
};
pub use logger::new_logger;
pub use project::import::import_canister_definitions;
pub use project::network_mappings::get_network_mappings;

// for nns
pub use project::import::{
    get_canisters_json_object, set_remote_canister_ids, ImportNetworkMapping,
};
