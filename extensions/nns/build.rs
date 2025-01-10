use std::env;
use std::path::PathBuf;

const REPLICA_REV: &str = "760e1f764b56f4f655a09789c245da487eccc5cb";

const BINARY_DEPENDENCIES: &[(&str, &str)] = &[
    // (downloaded binary name, renamed binary name)
    ("ic-nns-init", "ic-nns-init"),
    ("ic-admin", "ic-admin"),
    ("sns", "sns-cli"),
];

fn main() {
    // keep copy of the dependency in the root of the project, so that cargo-dist will be able to package it into a tarball
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // and also in `target/debug` or `target/release` for development purposes (e.g. cargo run), this is a bit hacky: https://github.com/rust-lang/cargo/issues/9661
    let target_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    for (binary_name, renamed_binary_name) in BINARY_DEPENDENCIES {
        let bin_in_manifest_dir = manifest_dir.join(renamed_binary_name);
        let bin_in_target_dir = target_dir.join(renamed_binary_name);
        dbg!(&bin_in_manifest_dir, &bin_in_target_dir);
        dfx_extensions_utils::download_ic_binary(REPLICA_REV, binary_name, &bin_in_manifest_dir);
        if bin_in_target_dir.exists() {
            std::fs::remove_file(&bin_in_target_dir).unwrap();
        }
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::copy(&bin_in_manifest_dir, &bin_in_target_dir).unwrap();
    }
}
