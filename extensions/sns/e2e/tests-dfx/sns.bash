#!/usr/bin/env bats

load "$(git rev-parse --show-toplevel)"/e2e/e2e_utils.sh

setup() {
    standard_setup

    dfx extension install sns
}

teardown() {
    dfx_stop

    standard_teardown
}

# The location of the SNS init config.
SNS_CONFIG_FILE_NAME="sns.yml"

@test "sns config create and validate fail outside of a project" {
    run dfx extension run sns config create
    assert_failure
    assert_output --partial 'Error: No config file found. Please run `dfx config create` first.'
    run dfx extension run sns config validate
    assert_failure
    assert_output --partial 'Error: No config file found. Please run `dfx config create` first.'
}

@test "sns config create creates a default configuration" {
    dfx_new
    run dfx extension run sns config create
    assert_success
    assert_output --regexp "Created SNS configuration at: .*/sns.yml"
    : "Check that the file exists..."
    test -e sns.yml
}

@test "sns config validate approves a valid configuration" {
    dfx_new
    install_asset sns/valid
    run dfx extension run sns config validate
    assert_success
    assert_output --partial 'SNS config file is valid'
}

@test "sns config validate identifies a missing key" {
    dfx_new
    install_asset sns/valid
    grep -v token_name "${SNS_CONFIG_FILE_NAME}" | sponge "$SNS_CONFIG_FILE_NAME"
    run dfx extension run sns config validate
    assert_failure
    assert_output --partial "Error: token-name must be specified"

}

@test "sns deploy exists" {
    dfx extension run sns deploy --help
}

@test "sns deploy fails without config file" {
    dfx_new
    dfx extension install nns
    dfx extension run nns import
    rm -f sns.yml # Is not expected to be present anyway
    run dfx extension run sns deploy
    assert_failure
    assert_output --partial "Error encountered when generating the SnsInitPayload: Couldn't open initial parameters file"
}

@test "sns deploy succeeds" {
    dfx_new
    install_shared_asset subnet_type/shared_network_settings/system
    dfx start --clean --background --host 127.0.0.1:8080
    sleep 1
    dfx extension install nns
    dfx extension run nns install
    dfx extension run nns import
    dfx extension run sns import
    ls candid
    cat dfx.json
    # Deploy the SNS
    install_asset sns/valid
    dfx extension run sns config validate
    # The remaining steps don't work any more as a pre-launch whitelist has been added.
    #dfx sns deploy
    # SNS canister IDs should be saved
    #dfx canister id sns_governance
    #dfx canister id sns_index
    #dfx canister id sns_ledger
    #dfx canister id sns_root
    #dfx canister id sns_swap
}
