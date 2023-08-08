use std::path::PathBuf;

const REPLICA_REV: &str = "90e2799c255733409d0e61682685afcc2431c928";

const BINARY_DEPENDENCIES: &[(&str, &str)] = &[
    // (downloaded binary name, renamed binary name)
    ("sns", "sns-cli"),
];

fn main() {
    // keep copy of the dependency in the root of the project, so that cargo-dist will be able to package it into a tarball
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // and also in `target/debug` or `target/release` for development purposes (e.g. cargo run)
    let target_dir = manifest_dir
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join(std::env::var("PROFILE").unwrap());
    for (binary_name, renamed_binary_name) in BINARY_DEPENDENCIES {
        let destination_paths = (
            manifest_dir.join(renamed_binary_name),
            target_dir.join(renamed_binary_name),
        );
        dbg!(&destination_paths);
        dfx_extensions_utils::download_ic_binary(REPLICA_REV, binary_name, &destination_paths.0);
        if destination_paths.1.exists() {
            std::fs::remove_file(&destination_paths.1).unwrap();
        }
        std::fs::create_dir_all(destination_paths.1.parent().unwrap()).unwrap();
        std::fs::copy(destination_paths.0, destination_paths.1).unwrap();
    }
}
