<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.5.0] - 2025-02-04
- Fixed a bug where `dfx nns install` and `dfx nns import` would fail if a canister type in dfx.json was defined by an extension.
- Added support for application subnets for `dfx start --pocketic`.
- Configure the PocketIC SNS subnet in the SNS-W canister and the default subnet in the CMC canister.
- No breaking changes since 0.4.7, but bumping the minor version number to keep it in sync with the SNS extension.

## [0.4.7] - 2024-11-08
- Added support for `dfx start --pocketic`.

## [0.4.6] - 2024-10-10
- Unchanged from 0.4.5

## [0.4.5] - 2024-09-20
- Updated the version of IC canisters used internally, as the previous version had removed support for some NNS proposals that the extension needed internally.

## [0.4.4] - 2024-09-12
- Updated version of ic-admin used internally

## [0.4.3] - 2024-07-05
- Unchanged from 0.4.2

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
[0.5.0]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.7]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.6]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.5]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.4]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.3]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.2]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.1]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.3.1...{{tag_name}}
[0.3.1]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.3.0...nns-v0.3.1
[0.3.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.2.1...nns-v0.3.0
[0.2.1]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.2.0...nns-v0.2.1
[0.2.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.1.0...nns-v0.2.0
[0.1.0]: https://github.com/dfinity/dfx-extensions/compare/nns-v0.1.0...nns-v0.1.0
