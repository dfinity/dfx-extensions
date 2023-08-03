#!/usr/bin/env bats

export GIT_ROOT_DIR="$(git rev-parse --show-toplevel)"
export CARGO_HOME="$HOME"

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
    assert_output --regexp "Error encountered when generating the SnsInitPayload.* Couldn't open initial parameters file .*sns.yml.* No such file or directory"
}

@test "sns deploy succeeds" {
    dfx_extension_install_manually nns
    dfx_new
    install_shared_asset subnet_type/shared_network_settings/system
    dfx start --clean --background --host 127.0.0.1:8080
    sleep 1
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
