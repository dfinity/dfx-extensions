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
SNS_CONFIG_FILE_V2_NAME="sns_v2.yml"

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

     dfx_new_frontend && dfx deploy
     BACKEND_CANISTER=$(dfx canister id e2e_project_backend)
     cat dfx.json
     FRONTEND_CANISTER=$(dfx canister id e2e_project_frontend)

     run dfx sns prepare-canisters add-nns-root "${BACKEND_CANISTER}" "${FRONTEND_CANISTER}"
     assert_success

     run dfx canister info "${BACKEND_CANISTER}"
     # Assert that the NNS Root canister (hard-coded ID) was actually added
     assert_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"

     run dfx canister info "${FRONTEND_CANISTER}"
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

     dfx_new && dfx deploy
     BACKEND_CANISTER=$(dfx canister id e2e_project_backend)
     FRONTEND_CANISTER=$(dfx canister id e2e_project_frontend)

     run dfx sns prepare-canisters add-nns-root "${BACKEND_CANISTER}" "${FRONTEND_CANISTER}"
     assert_success

     run dfx canister info "${BACKEND_CANISTER}"
     # Assert that the NNS Root canister (hard-coded ID) was actually added
     assert_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"

     run dfx canister info "${FRONTEND_CANISTER}"
     # Assert that the NNS Root canister (hard-coded ID) was actually added
     assert_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"

     run dfx sns prepare-canisters remove-nns-root  "${BACKEND_CANISTER}" "${FRONTEND_CANISTER}"
     assert_success

     run dfx canister info "${BACKEND_CANISTER}"
     # Assert that the NNS Root canister (hard-coded ID) was actually removed
     refute_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"

     run dfx canister info "${FRONTEND_CANISTER}"
     # Assert that the NNS Root canister (hard-coded ID) was actually removed
     refute_output --partial "r7inp-6aaaa-aaaaa-aaabq-cai"
}

# This test asserts that the `propose` subcommand exist in the current extension version.
@test "sns propose exists" {
    run dfx sns propose --help
    assert_output --partial "dfx sns propose"
}

# This test asserts that at least one neuron flag must be specified to succeed
@test "sns propose must use a neuron flag" {
    install_asset sns/valid

    run dfx sns propose sns_v2.yml
    assert_failure
    assert_output --partial "sns propose --dfx-cache-path <DFX_CACHE_PATH> <--neuron-id <NEURON_ID>|--neuron-memo <NEURON_MEMO>|--test-neuron-proposer> <INIT_CONFIG_FILE>"
}

# This test asserts that a local dfx server with the NNS installed can submit a
# CreateServiceNervousSystem NNS Proposal with the test neuron
@test "sns propose can submit a proposal with the test neuron" {
    dfx_new

    dfx_extension_install_manually nns
    install_shared_asset subnet_type/shared_network_settings/system
    install_asset sns/valid

    dfx_start_for_nns_install
    dfx nns install

    run dfx sns propose --test-neuron-proposer "${SNS_CONFIG_FILE_V2_NAME}"
    assert_success
    assert_output --partial "🚀 Success!"
    assert_output --partial "Proposal ID"
}

# This test asserts that a local dfx server wih the NNS installed can a
# CreateServiceNervousSystem NNS Proposal with the --neuron-id flag,
# which requires actual staking of a neuron in the NNS.
@test "sns propose can submit a proposal with neuron id" {
    dfx_new

    dfx_extension_install_manually nns
    install_shared_asset subnet_type/shared_network_settings/system
    install_asset sns

    dfx_start_for_nns_install
    dfx nns import
    dfx nns install

    # Import the identity we'll use for the tests
    dfx identity import --force --disable-encryption ident-1 ident-1/identity.pem
    dfx identity use ident-1

    # Transfer the stake required to create a neuron
    run dfx ledger transfer --amount 10 --memo 0 a749bfc34e8f202046e9c836f46c23a327dbf78fe223cf4a893a59ed60dd1883
    assert_success

    # Create the Neuron and extract the Neuron Id
    RESPONSE=$(dfx canister call nns-governance claim_or_refresh_neuron_from_account '(record {
        controller = opt principal "hpikg-6exdt-jn33w-ndty3-fc7jc-tl2lr-buih3-cs3y7-tftkp-sfp62-gqe";
        memo = 0 : nat64;
    })')
    NEURON_ID=$(echo "${RESPONSE}" | awk -F 'id = ' '{print $2}' | cut -d ' ' -f 1 | tr -d '[:space:]_')

    # Extend its dissolve delay to 6 months so it can submit proposals
    run dfx canister call nns-governance  manage_neuron "(record {
        id = opt record { id = $NEURON_ID : nat64 };
        command = opt variant {
            Configure = record {
                operation = opt variant {
                    IncreaseDissolveDelay = record {
                        additional_dissolve_delay_seconds = 31_622_400 : nat32;
                    }
                };
            }
        };
        neuron_id_or_subaccount = null;
    })"
    assert_success

    # Actually submit the proposal
    run dfx sns propose --neuron-id "${NEURON_ID}" "valid/${SNS_CONFIG_FILE_V2_NAME}"
    assert_success
    assert_output --partial "🚀 Success!"
    assert_output --partial "Proposal ID"
}
