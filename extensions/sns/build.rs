use std::path::Path;

const REPLICA_REV: &str = "90e2799c255733409d0e61682685afcc2431c928";

const BINARY_DEPENDENCIES: &[(&str, &str)] = &[
    // (downloaded binary name, renamed binary name)
    ("sns", "sns-cli"),
];

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    for (binary_name, renamed_binary_name) in BINARY_DEPENDENCIES {
        let destination_path = Path::new(&manifest_dir).join(renamed_binary_name);
        dfx_extensions_utils::download_ic_binary(REPLICA_REV, binary_name, &destination_path);
    }
}
