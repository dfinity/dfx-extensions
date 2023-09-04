# Contributing

## How it works 

The `dfx` utility offers a feature to install new extensions. This process ensures that the extensions are not only installed from a trusted source but are also compatible with the user's dfx version, enhancing the utility's reliability and user experience. Here's a high-level overview of how the installation process works:

1. Determine Extension Compatibility:
    - Before any installation can take place, `dfx` checks if the requested extension is already installed. If it is, the process terminates with an error.
    - `dfx` identifies the version of the extension compatible with its own version. This ensures that users don't end up installing extensions that may not work properly with their specific version of `dfx`. Compatibility is determined using an extension compatibility matrix (`compatibility.json` in the root of this repository), which maps extensions to compatible versions of `dfx`.

2. Download and Extraction:
    - Once the compatible version of the extension is determined, `dfx` constructs a download URL. This URL points to a GitHub releases page where the extensions are hosted.
    - The extension is then downloaded from this URL. 
    - After a successful download, the extension, which is in a compressed archive format (.tar.gz), is unpacked to a temporary directory. Post extraction, the extension is renamed and moved to its permanent location (the directory where extensions are meant to reside).

## Testing 

### Unit testing 

```console
cargo test --workspace 
```

### End-to-end testing (e2e)

1. Make sure the repository's git submodules are downloaded:
    - When cloning the `dfx-extensions` repository:
    ```console
    git clone --recursive https://github.com/dfinity/dfx-extensions
    ```
    - If you have already cloned the `dfx-extensions` repository:
    ```console
    cd dfx-extensions
    git submodule update --init --recursive
    ```

2. Run e2e tests:
   ```console
   e2e/bats/bin/bats extensions/**/e2e/**/*.bash
   ```

## Release a new version of the extension
Once your changes are on the `main` branch and you wish to release a new version of the extension, the process consists of the following steps:
1. Create a new GitHub Release with the extension's binaries. The whole process is automated using GitHub Actions. Go to https://github.com/dfinity/dfx-extensions/actions/workflows/release-with-github.yml and click `Run workflow`, then follow the instructions in the pop-up menu. Alternatively, you can trigger this workflow from your shell by navigating to the repository directory and running:
    ```console 
    gh workflow run "release-with-github.yml" -f whichCrate=nns -f semverVersion=1.2.3
    ```
Once the workflow finishes running, it will open a PR that you need to review and merge.
2. Copy the relevant section from the extension's `CHANGELOG.md` to the description in GitHub Releases.

## Styleguides

### Git Commit Messages

For formatting commit messages, follow the guidelines from [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
