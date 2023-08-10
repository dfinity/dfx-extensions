#!/usr/bin/env bats

export GIT_ROOT_DIR="$(git rev-parse --show-toplevel)"

load "$GIT_ROOT_DIR"/e2e/utils.sh

setup() {
    standard_setup

    dfx_extension_install_manually sns
}

teardown() {
    dfx_stop

    standard_teardown
}

# The location of the SNS init config.
SNS_CONFIG_FILE_NAME="sns.yml"

@test "sns-cli binary exists and is executable" {
    run "$(dfx cache show)"/extensions/sns/sns-cli --help
    assert_output --partial "Initialize, deploy and interact with an SNS"
}

@test "sns config create and validate fail outside of a project" {
    run dfx sns config create
    assert_failure
    assert_output --partial 'Error: Cannot find dfx configuration file in the current working directory. Did you forget to create one?'
    run dfx sns config validate
    assert_failure
    assert_output --partial 'Error: Cannot find dfx configuration file in the current working directory. Did you forget to create one?'
}

@test "sns config create creates a default configuration" {
    dfx_new
    run dfx sns config create
    assert_success
    assert_output --regexp "Created SNS configuration at: .*/sns.yml"
    : "Check that the file exists..."
    test -e sns.yml
}

@test "sns config validate approves a valid configuration" {
    dfx_new
    install_asset sns/valid
    run dfx sns config validate
    assert_success
    assert_output --partial 'SNS config file is valid'
}

@test "sns config validate identifies a missing key" {
    dfx_new
    install_asset sns/valid
    grep -v token_name "${SNS_CONFIG_FILE_NAME}" | sponge "$SNS_CONFIG_FILE_NAME"
    run dfx sns config validate
    assert_failure
    assert_output --partial "Error: token-name must be specified"

}

@test "sns deploy exists" {
    dfx sns deploy --help
}

@test "sns deploy fails without config file" {
    dfx_extension_install_manually nns
    dfx_new
    dfx nns import
    rm -f sns.yml # Is not expected to be present anyway
    run dfx sns deploy
    assert_failure
    assert_output --regexp "Error encountered when generating the SnsInitPayload: Unable to read .*sns.yml.* No such file or directory"
}

@test "sns deploy succeeds" {
    dfx_extension_install_manually nns
    dfx_new
    install_shared_asset subnet_type/shared_network_settings/system
    dfx start --clean --background --host 127.0.0.1:8080
    wait_until_replica_healthy
    dfx nns install
    dfx nns import
    dfx sns import
    ls candid
    cat dfx.json
    # Deploy the SNS
    install_asset sns/valid
    dfx sns config validate
    # The remaining steps don't work any more as a pre-launch whitelist has been added.
    #dfx sns deploy
    # SNS canister IDs should be saved
    #dfx canister id sns_governance
    #dfx canister id sns_index
    #dfx canister id sns_ledger
    #dfx canister id sns_root
    #dfx canister id sns_swap
}

# This test asserts that the `prepare-canisters` subcommand and it's child subcommands
# exist in the current extension version.
@test "sns prepare-canisters exists" {
    run dfx sns prepare-canisters --help
    assert_output --partial "dfx sns prepare-canisters"
    run dfx sns prepare-canisters add-nns-root --help
    assert_output --partial "dfx sns prepare-canisters add-nns-root"
    run dfx sns prepare-canisters remove-nns-root --help
    assert_output --partial "dfx sns prepare-canisters remove-nns-root"
}

# This test asserts that the new subcommand `prepare-canister add-nns-root` can add NNS root
# as a co-controller to a dapp.
@test "sns prepare-canisters adds NNS Root" {
     dfx_extension_install_manually nns
     install_shared_asset subnet_type/shared_network_settings/system
     dfx start --clean --background --host 127.0.0.1:8080
     wait_until_replica_healthy

     WALLET_CANISTER_ID=$(dfx identity get-wallet)

     # TODO specify multiple
     run dfx sns prepare-canisters add-nns-root "${WALLET_CANISTER_ID}"
     assert_success

     run dfx canister info "${WALLET_CANISTER_ID}"
     # Assert that the NNS Root canister (hard-coded ID) was actually added
     assert_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"

     run dfx canister info "${NEW_CANISTER_ID}"
     # Assert that the NNS Root canister (hard-coded ID) was actually added
     assert_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"
}

# This test asserts that the new subcommand `prepare-canister remove-nns-root` can remove NNS root
# as a co-controller to a dapp.
@test "sns prepare-canisters removes NNS Root" {
     dfx_extension_install_manually nns
     install_shared_asset subnet_type/shared_network_settings/system
     dfx start --clean --background --host 127.0.0.1:8080
     wait_until_replica_healthy

     WALLET_CANISTER_ID=$(dfx identity get-wallet)

     run dfx sns prepare-canisters add-nns-root "${WALLET_CANISTER_ID}"
     assert_success

     run dfx canister info "${WALLET_CANISTER_ID}"
     # Assert that the NNS Root canister (hard-coded ID) was actually added
     assert_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"

     run dfx sns prepare-canisters remove-nns-root  "${WALLET_CANISTER_ID}"
     assert_success

     run dfx canister info "${WALLET_CANISTER_ID}"
     # Assert that the NNS Root canister (hard-coded ID) was actually removed
     refute_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"
}
