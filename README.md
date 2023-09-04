# DFINITY `dfx` Extensions Repository

## Introduction
Welcome to the `dfx` extensions repository by DFINITY. This repository provides essential extensions to the `dfx` command-line tool, aiming to enhance its functionality.

## Features

- Extend `dfx` functionality with custom commands.
- Comprehensive end-to-end tests ensuring extensions' robustness.
- Extensions sourced from DFINITY-reviewed code.
- Metadata file `extension.json` for smooth user experience.

## Repository Structure
The repository has the following main directories:

- **e2e:** Contains essential utilities for end-to-end testing of extensions.
- **extensions:** Houses all the individual extensions. Currently, we have:
  - **nns:** The NNS (Network Nervous System) extension with its configuration files, tests, commands, and source code.
  - **sns:** The SNS (Secondary Nervous System) extension with its configuration files, tests, commands, and source code.
- **extensions-utils:** Shared utilities for all extensions, including dependency management, error handling, logging, and project handling. You may find them useful if you're building your own extensions.

Each extension in the DFINITY extensions repository has its structure:

- README.md: An overview of the extension.
- CHANGELOG.md: Tracks changes in different versions of the extension.
- e2e: End-to-end tests and associated assets.
- extension.json: Metadata about the extension.
- src: Contains the Rust source code for the extension.

## Getting Started

### Installing/Uninstalling/Upgrading Extensions
As the extensions are built with Rust, you can follow the usual Rust procedures to build, install, and manage them.

### Extension Development
1. Create a separate directory under the `extensions` folder for your extension.
2. Provide a manifest (`extension.json`) with metadata for your extension.
3. Ensure compatibility with specific `dfx` versions, which can be checked using the `compatibility.json` file.

## Compatibility
Extensions should be compatible with specific `dfx` versions. The compatibility matrix can be found in the `compatibility.json` file.

## Contribution
Contributions are welcome! If you wish to enhance an existing extension or add a new one, please follow the structure as outlined in the repository. For any significant changes, consider creating an issue or pull request.

If you're looking to contribute, especially regarding releasing extensions, updating compatibility files, or any other best practices, please refer to the [CONTRIBUTING.md](CONTRIBUTING.md) file for detailed guidelines.

## License
This project is licensed under the [MIT License](LICENSE).
