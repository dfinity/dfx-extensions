pub mod nns;
pub mod sns;

use anyhow::Context;
use dfx_core::fs;
use flate2::read::GzDecoder;
use fn_error_context::context;

use url::Url;

use std::path::{Component, Path};

/// Downloads a file (this function should be used for canister modules)
#[context("Failed to download '{:?}' from '{:?}'.", target, source.as_str())]
pub async fn download_gz(source: &Url, target: &Path) -> anyhow::Result<()> {
    download_gz_and_maybe_ungzip(source, target, false).await
}

/// Downloads and unzips a file (this function should be used for x86 binaries)
#[context("Failed to download and unzip '{:?}' from '{:?}'.", target, source.as_str())]
pub async fn download_gz_and_ungzip(source: &Url, target: &Path) -> anyhow::Result<()> {
    download_gz_and_maybe_ungzip(source, target, true).await
}

pub async fn download_gz_and_maybe_ungzip(
    source: &Url,
    target: &Path,
    unzip: bool,
) -> anyhow::Result<()> {
    if target.exists() {
        println!("Already downloaded: {}", target.to_string_lossy());
        return Ok(());
    }
    println!(
        "Downloading {}\n  from .gz: {}",
        target.to_string_lossy(),
        source.as_str()
    );
    let response = reqwest::get(source.clone())
        .await
        .with_context(|| "Failed to connect")?
        .bytes()
        .await
        .with_context(|| "Download was interrupted")?;

    let target_parent = target
        .parent()
        .unwrap_or_else(|| Path::new(Component::CurDir.as_os_str()));
    let tmp_dir = tempfile::TempDir::new_in(target_parent)
        .with_context(|| "Failed to create temporary directory for download")?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join("wasm");
        let mut file = std::fs::File::create(&filename).with_context(|| {
            format!(
                "Failed to write temp file when downloading '{}'.",
                filename.display()
            )
        })?;
        if unzip {
            let mut decoder = GzDecoder::new(&response[..]);
            std::io::copy(&mut decoder, &mut file)
                .with_context(|| format!("Failed to unzip WASM to '{}'", filename.display()))?;
        } else {
            std::io::copy(&mut response.as_ref(), &mut file)
                .with_context(|| format!("Failed to unzip WASM to '{}'", filename.display()))?;
        }
        filename
    };
    fs::rename(&downloaded_filename, target).with_context(|| {
        format!(
            "Failed to move downloaded tempfile '{}' to '{}'.",
            downloaded_filename.display(),
            target.display()
        )
    })?;
    Ok(())
}

/// Downloads wasm file from the main IC repo CI.
#[context("Failed to download {} from the IC CI.", wasm_name)]
pub async fn download_ic_repo_wasm(
    wasm_name: &str,
    ic_commit: &str,
    wasm_dir: &Path,
) -> anyhow::Result<()> {
    fs::create_dir_all(wasm_dir)
        .with_context(|| format!("Failed to create wasm directory: '{}'", wasm_dir.display()))?;
    let final_path = wasm_dir.join(wasm_name);
    let url_str =
        format!("https://download.dfinity.systems/ic/{ic_commit}/canisters/{wasm_name}.gz");
    let url = Url::parse(&url_str)
      .with_context(|| format!("Could not determine download URL. Are ic_commit '{ic_commit}' and wasm_name '{wasm_name}' valid?"))?;
    download_gz(&url, &final_path).await
}
