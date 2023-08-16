use backoff::future::retry;
use backoff::ExponentialBackoffBuilder;
use flate2::read::GzDecoder;
use std::path::Path;
use std::time::Duration;
use std::{fs, io::copy};
use tokio::runtime::Runtime;

pub fn download_ic_binary(replica_rev: &str, binary_name: &str, destination_path: &Path) {
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "x86_64", // let's rely on rosetta2 for now, since ic binaiers are not available for arm64
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

    let bytes = Runtime::new().unwrap().block_on(download_bytes(&url));
    let mut d = GzDecoder::new(&*bytes);
    let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
    copy(&mut d, &mut temp_file).expect("Failed to copy content");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        dfx_core::fs::set_permissions(&temp_file.path(), std::fs::Permissions::from_mode(0o500))
            .expect("Failed to set permissions");
    }
    fs::rename(temp_file, destination_path).expect("Failed to move extension");
}

async fn download_bytes(url: &str) -> Vec<u8> {
    let retry_policy = ExponentialBackoffBuilder::new()
        .with_initial_interval(Duration::from_secs(1))
        .with_max_interval(Duration::from_secs(16))
        .with_multiplier(2.0)
        .with_max_elapsed_time(Some(Duration::from_secs(300)))
        .build();
    let resp = retry(retry_policy, || async {
        match reqwest::get(url).await {
            Ok(response) => Ok(response),
            Err(err) => Err(backoff::Error::transient(err)),
        }
    })
    .await
    .unwrap();

    let bytes = resp.bytes().await.expect("Failed to read response");
    bytes.to_vec()
}
