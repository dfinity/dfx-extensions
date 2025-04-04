<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.5.1] - 2025-04-04

## [0.5.0] - 2025-02-04
- Improved the behavior of SNS Testflight. There is currently an incompatibilty between the `sns-init.yaml` format and SNS Testflight. [Refer to this forum post](https://forum.dfinity.org/t/error-when-deploying-sns-testflight-to-mainnet/38282/6?u=andre-popovitch) for more information and workarounds.
- Added `UpgradeSnsControlledCanister` command.
- The SNS extension will now use your DFX identity and settings. One consequence of this is that it defaults to the local network when it can (similar to the behavior of DFX itself).

## [0.4.7] - 2024-11-08
- Fixed a bug where `dfx sns import` would fail if a canister type in dfx.json was defined by an extension.
- Added support for `dfx start --pocketic`.

## [0.4.6] - 2024-10-11
- Added `dfx sns health` command.

## [0.4.5] - 2024-09-20
- Unchanged from v0.4.4

## [0.4.4] - 2024-09-12
- Add `dfx sns list` command to view available SNSes

## [0.4.3] - 2024-07-05
- `dfx sns download` now downloads the index-ng canister, which is the version needed for SNS testflight

## [0.4.2] - 2024-07-02
- Added the `neuron-id-to-candid-subaccount` command.
- Added a warning/confirmation text to the `propose` command.

## [0.4.1] - 2024-05-29
- The `Principals` field of sns-init.yaml is no longer required.

## [0.4.0] - 2024-04-03

- The behavior of the `dfx sns` extension has been updated to match the `sns-cli` tool.
  The main effect of this is that many deprecated commands have been removed, and the remaining commands have been renamed.
- Renamed `dfx sns config` to `dfx sns init-config-file`
- Removed `dfx sns config create`. Instead, check the [sns-testing repo](https://github.com/dfinity/sns-testing/blob/main/example_sns_init.yaml) for an example template you can base your `sns_init.yaml` on.
- Removed `dfx sns deploy`
- Introduced `dfx sns deploy-testflight`, which can be used to create a test deployment of the SNS on mainnet or on a local replica.

## [0.3.1] - 2024-02-09
- Same functionality as version `0.3.0`.

## [0.3.0] - 2024-02-07

- Same functionality as version `0.2.1`.
- Updated SNS canisters to the latest version.

## [0.2.1] - 2023-08-29

- Same functionality as version `0.2.0`.
- Improved compatibility: Binaries for Linux were built using `ubuntu-20.04`, which enhances compatibility with older `libc` versions.

## [0.2.0] - 2023-08-16

- Introduced support for a new filenaming scheme for the tarball. See [PR #3307](https://github.com/dfinity/sdk/pull/3307).
- **Note**: This version was retracted and superseded by `0.2.1`.

### Added
- Add the new sns extension subcommand `prepare-canisters`.
- Add the new sns extension subcommand `propose`.

## [0.1.0] - 2023-07-12

- Lifted the functionality of the `dfx sns` command from the SDK repository and integrated it into the `dfx-extension`. More details in [commit 4b2a8916c3362ec18aea43f8085dc441c3be2b9d](https://github.com/dfinity/sdk/commit/4b2a8916c3362ec18aea43f8085dc441c3be2b9d).

## [0.1.0] - 2023-07-12

<!-- next-url -->
[Unreleased]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...HEAD
[0.5.1]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.5.0]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.8]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.7]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.6]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.5]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.4]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.3]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.2]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.1]: https://github.com/dfinity/dfx-extensions/compare/{{tag_name}}...{{tag_name}}
[0.4.0]: https://github.com/dfinity/dfx-extensions/compare/sns-v0.3.1...{{tag_name}}
[0.3.1]: https://github.com/dfinity/dfx-extensions/compare/sns-v0.3.0...sns-v0.3.1
[0.3.0]: https://github.com/dfinity/dfx-extensions/compare/sns-v0.2.1...sns-v0.3.0
[0.2.1]: https://github.com/dfinity/dfx-extensions/compare/sns-v0.2.0...sns-v0.2.1
[0.2.0]: https://github.com/dfinity/dfx-extensions/compare/sns-v0.1.0...sns-v0.2.0
[0.1.0]: https://github.com/dfinity/dfx-extensions/compare/sns-v0.1.0...sns-v0.1.0
