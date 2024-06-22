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
SNS_CONFIG_FILE_NAME="sns_init.yaml"

@test "sns init-config-file validate approves a valid configuration" {
    dfx_new
    install_asset sns/valid
    run dfx sns init-config-file validate
    assert_success
    assert_output '' # no output if the file is valid
}

@test "sns init-config-file validate identifies a missing key" {
    dfx_new
    install_asset sns/valid
    # make the config file invalid by removing lines that contain "transaction_fee"
    # every test is run in a unique temporary directory, so we aren't modifying
    # anything that will be used by other tests by doing this.
    grep -v transaction_fee "${SNS_CONFIG_FILE_NAME}" | sponge "$SNS_CONFIG_FILE_NAME"
    run dfx sns init-config-file validate
    assert_failure
    assert_output --partial "transaction_fee"
}

@test "sns propose exists" {
    run dfx sns propose --help
    assert_output --partial "Subcommand for submitting a CreateServiceNervousSystem NNS Proposal"
}

@test "sns propose fails without config file" {
    dfx_extension_install_manually nns
    dfx_new
    dfx nns import
    rm -f sns.yml # Is not expected to be present anyway
    run dfx sns propose --neuron-id 1
    assert_failure
    assert_output --partial "Unable to read the SNS configuration file"
}

@test "sns propose succeeds" {
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
    dfx sns init-config-file validate
    # The remaining steps don't work any more as the steps required have changed due to one-proposal
    #dfx sns propose
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

     dfx_new_frontend && dfx deploy
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
@test "sns deploy-testflight exists" {
    run dfx sns deploy-testflight --help
    assert_output --partial "Deploy an sns directly to a local replica or the Internet Computer"
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
    run dfx sns propose --neuron-id "${NEURON_ID}" "valid/${SNS_CONFIG_FILE_NAME}" --skip-confirmation
    assert_success
    assert_output --partial "🚀 Success!"
    assert_output --partial "Proposal ID"
}

# This test asserts that the `neuron-id-to-candid-subaccount` subcommand exist in the current extension version.
@test "sns neuron-id-to-candid-subaccount exists" {
    run dfx sns neuron-id-to-candid-subaccount --help
    assert_output --partial "Converts a Neuron ID to a candid subaccount blob"
}
# check the output of a particular case of neuron-id-to-candid-subaccount
@test "sns neuron-id-to-candid-subaccount has a reasonable output" {
    run dfx sns neuron-id-to-candid-subaccount 9f5f9fda77a03e7177126d0be8c99e931a5381731d00da53ede363140e1be5d6
    assert_output 'blob "\9f\5f\9f\da\77\a0\3e\71\77\12\6d\0b\e8\c9\9e\93\1a\53\81\73\1d\00\da\53\ed\e3\63\14\0e\1b\e5\d6"'
}
@test "sns neuron-id-to-candid-subaccount --escaped has a reasonable output" {
    run dfx sns neuron-id-to-candid-subaccount 9f5f9fda77a03e7177126d0be8c99e931a5381731d00da53ede363140e1be5d6 --escaped
    assert_output 'blob \"\\9f\\5f\\9f\\da\\77\\a0\\3e\\71\\77\\12\\6d\\0b\\e8\\c9\\9e\\93\\1a\\53\\81\\73\\1d\\00\\da\\53\\ed\\e3\\63\\14\\0e\\1b\\e5\\d6\"'
}