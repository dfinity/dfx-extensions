use flate2::read::GzDecoder;
use std::fs::File;
use std::path::Path;
use std::{env, fs, io::copy};

pub fn download_ic_binary(replica_rev: &str, binary_name: &str, renamed_binary_name: &str) {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "x86_64", // let's rely on rosetta2 for now, since sns-cli is not available for arm64
        _ => panic!("Unsupported architecture"),
    };
    let os = match std::env::consts::OS {
        "macos" => "darwin",
        "linux" => "linux",
        // "windows" => "windows", // unsupported till dfx supports windows
        _ => panic!("Unsupported OS"),
    };

    let url = format!(
        "https://download.dfinity.systems/ic/{replica_rev}/openssl-static-binaries/{arch}-{os}/{binary_name}.gz",
        arch = arch,
        os = os,
        binary_name = binary_name,
    );
    println!("Downloading {}", url);

    let resp = reqwest::blocking::get(&url).expect("Request failed");
    let bytes = resp.bytes().expect("Failed to read response");

    let mut d = GzDecoder::new(&*bytes);
    let mut dest = File::create(binary_name).expect("Failed to create sns file");

    copy(&mut d, &mut dest).expect("Failed to copy content");

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_path = Path::new(&manifest_dir).join(renamed_binary_name);
    fs::rename(binary_name, out_path).expect("Failed to move sns");
}
