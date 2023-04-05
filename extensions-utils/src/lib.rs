//! Library for calling bundled command line tools.

mod dfx;
mod download_wasms;
mod error;
mod logger;
mod project;

pub use dfx::{call_bundled, dfx_version, replica_rev, webserver_port, Cache};
pub use download_wasms::download_ic_repo_wasm;
pub use download_wasms::nns::download_nns_wasms;
pub use download_wasms::nns::{
    nns_wasm_dir, IcNnsInitCanister, StandardCanister, ED25519_TEST_ACCOUNT, NNS_CORE,
    NNS_FRONTEND, NNS_SNS_WASM, SECP256K1_TEST_ACCOUNT,
};
pub use download_wasms::sns::SnsCanisterInstallation;
pub use download_wasms::sns::{download_sns_wasms, SNS_CANISTERS};
pub use logger::new_logger;
pub use project::import::import_canister_definitions;
pub use project::network_mappings::get_network_mappings;

// for nns
pub use project::import::{
    get_canisters_json_object, set_remote_canister_ids, ImportNetworkMapping,
};
