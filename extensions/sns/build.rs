const REPLICA_REV: &str = "0062aec2efc16d6e4cadb2cd1052aaabbc9f6e48";

const BINARY_DEPENDENCIES: &[(&str, &str)] = &[
    // (downloaded binary name, renamed binary name)
    ("sns", "sns-cli"),
];

fn main() {
    for (binary_name, renamed_binary_name) in BINARY_DEPENDENCIES {
        dfx_extensions_utils::download_dependencies::download_ic_binary(
            REPLICA_REV,
            binary_name,
            renamed_binary_name,
        );
    }
}
