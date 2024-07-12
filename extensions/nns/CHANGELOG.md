<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.4.3] - 2024-07-05

## [0.4.2] - 2024-07-02
- Corrected name of the extension in metadata from "sns" to "nns".

## [0.4.1] - 2024-05-29
- Bump II and NNS dapp to their latest mainnet verions (II: release-2024-05-13; NNS-Dapp: proposal-129748) and install their dependencies (ICRC1 ledger and index for ckETH, ICP index, SNS aggregator).

## [0.4.0] - 2024-04-04
- Same functionality as version `0.3.1`.

## [0.3.1] - 2024-02-09
- `dfx nns install` now configures the cycles minting canister such that it plays nicely with the cycles ledger (which has to be installed separately).

## [0.3.0] - 2024-02-07

- Same functionality as version `0.2.1`.
- Updated NNS canisters to the latest version.

## [0.2.1] - 2023-08-29

- Same functionality as version `0.2.0`.
- Improved compatibility: Binaries for Linux were built using `ubuntu-20.04`, which enhances compatibility with older `libc` versions.

## [0.2.0] - 2023-08-16

- Introduced support for a new filenaming scheme for the tarball. See [PR #3307](https://github.com/dfinity/sdk/pull/3307).
- **Note**: This version was retracted and superseded by `0.2.1`.

## [0.1.0] - 2023-07-12

- Lifted the functionality of the `dfx nns` command from the SDK repository and integrated it into the `dfx-extension`. More details in [commit 4b2a8916c3362ec18aea43f8085dc441c3be2b9d](https://github.com/dfinity/sdk/commit/4b2a8916c3362ec18aea43f8085dc441c3be2b9d).

<!-- next-url -->
[Unreleased]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...HEAD
[0.4.3]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.2]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.1]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.3.1...{{tag_name}}
[0.3.1]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.3.0...nns-v0.3.1
[0.3.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.2.1...nns-v0.3.0
[0.2.1]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.2.0...nns-v0.2.1
[0.2.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.1.0...nns-v0.2.0
[0.1.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.1.0...nns-v0.1.0
