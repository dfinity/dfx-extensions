#!/usr/bin/env bats

export GIT_ROOT_DIR="$(git rev-parse --show-toplevel)"
load "$GIT_ROOT_DIR"/e2e/utils.sh

assets="$(dirname "$BATS_TEST_FILENAME")"/../assets

setup() {
    standard_setup

    dfx_extension_install_manually nns

    dfx_new
}

teardown() {
    stop_webserver

    dfx_stop

    standard_teardown
}

@test "ic-nns-init binary exists and is executable" {
    # it panics, but still shows help
    run "$(dfx cache show)/extensions/nns/ic-nns-init" --help
    assert_failure
    assert_output --partial "thread 'main' panicked"
    assert_output --partial "Illegal arguments:"
    assert_output --partial "ic-nns-init [OPTIONS]"

    # --version fails too
    run "$(dfx cache show)/extensions/nns/ic-nns-init" --version
    assert_failure
}

@test "ic-admin binary exists and is executable" {
    run "$(dfx cache show)/extensions/nns/ic-admin" --help
    assert_success
    assert_output --partial 'Common command-line options for `ic-admin`'
}

@test "dfx nns install command exists" {
    run dfx nns install --help
    assert_success
}


# The nns canisters should be installed without changing any of the developer's project files,
# so we cannot rely on `dfx canister id` when testing.  We rely on these hard-wired values instead:
nns_canister_id() {
    case "$1" in
    nns-registry)          echo "rwlgt-iiaaa-aaaaa-aaaaa-cai" ;;
    nns-governance)        echo "rrkah-fqaaa-aaaaa-aaaaq-cai" ;;
    nns-ledger)            echo "ryjl3-tyaaa-aaaaa-aaaba-cai" ;;
    nns-root)              echo "r7inp-6aaaa-aaaaa-aaabq-cai" ;;
    nns-cycles-minting)    echo "rkp4c-7iaaa-aaaaa-aaaca-cai" ;;
    nns-lifeline)          echo "rno2w-sqaaa-aaaaa-aaacq-cai" ;;
    nns-genesis-token)     echo "renrk-eyaaa-aaaaa-aaada-cai" ;;
    # Coming soon:
    #nns-ic-ckbtc-minter)   echo "qjdve-lqaaa-aaaaa-aaaeq-cai" ;;
    nns-sns-wasm)          echo "qaa6y-5yaaa-aaaaa-aaafa-cai" ;;
    internet_identity)     echo "qhbym-qaaaa-aaaaa-aaafq-cai" ;;
    nns-dapp)              echo "qsgjb-riaaa-aaaaa-aaaga-cai" ;;
    *)                     echo "ERROR: Unknown NNS canister '$1'." >&2
                           exit 1;;
    esac
}

assert_nns_canister_id_matches() {
    [[ "$(nns_canister_id "$1")" == "$(dfx canister id "$1")" ]] || {
       echo "ERROR: NNS canister ID mismatch for $1: $(nns_canister_id "$1") != $(dfx canister id "$1")"
       exit 1
    } >&2
}

@test "dfx nns import ids are as expected" {
    dfx nns import
    assert_nns_canister_id_matches nns-registry
    assert_nns_canister_id_matches nns-governance
    assert_nns_canister_id_matches nns-ledger
    assert_nns_canister_id_matches nns-root
    assert_nns_canister_id_matches nns-cycles-minting
    assert_nns_canister_id_matches nns-lifeline
    assert_nns_canister_id_matches nns-genesis-token
    # Coming soon:
    # assert_nns_canister_id_matches nns-ic-ckbtc-minter
    assert_nns_canister_id_matches nns-sns-wasm
    # TODO: No source provides these canister IDs - yet.
    #assert_nns_canister_id_matches internet_identity
    #assert_nns_canister_id_matches nns-dapp
}

@test "dfx nns install on application subnet" {
    echo Setting up...
    install_shared_asset subnet_type/shared_network_settings/application
    dfx_start_for_nns_install

    run dfx nns install

    assert_success

    SNS_SUBNET_ID=$(curl http://localhost:8080/_/topology | jq -r '.subnet_configs | map_values(select(.subnet_kind=="SNS")) | keys[]')
    if [[ "${SNS_SUBNET_ID}" == "" ]]
    then
      echo "No SNS subnet found in the PocketIC instance topology."
      exit 1
    fi
    run dfx canister call qaa6y-5yaaa-aaaaa-aaafa-cai get_sns_subnet_ids '(record {})' --query
    assert_success
    assert_output --partial "${SNS_SUBNET_ID}"

    APP_SUBNET_ID=$(curl http://localhost:8080/_/topology | jq -r '.subnet_configs | map_values(select(.subnet_kind=="Application")) | keys[]')
    if [[ "${APP_SUBNET_ID}" == "" ]]
    then
      echo "No application subnet found in the PocketIC instance topology."
      exit 1
    fi
    while [[ "$(dfx canister call rkp4c-7iaaa-aaaaa-aaaca-cai get_default_subnets '()' --query | grep "${APP_SUBNET_ID}")" == "" ]]
    do
      sleep 1
    done
    run dfx canister call rkp4c-7iaaa-aaaaa-aaaca-cai get_default_subnets '()' --query
    assert_success
    assert_output --partial "${APP_SUBNET_ID}"
}

@test "dfx nns install runs" {

    echo Setting up...
    install_shared_asset subnet_type/shared_network_settings/system
    dfx_start_for_nns_install
    dfx nns install

    echo "Checking that the install worked..."
    echo "   The expected wasms should be installed..."
    # Note:  The installation is quite expensive, so we test extensively on one installation
    #        rather than doing a separate installation for every test.  The tests are read-only
    #        so no test should affect the output of another.
    installed_wasm_hash() {
        dfx canister info "$(nns_canister_id "$1")" | awk '/Module hash/{print $3; exit 0}END{exit 1}'
    }
    downloaded_wasm_hash() {
        sha256sum "$DFX_CACHE_ROOT/.cache/dfinity/versions/$(dfx --version | awk '{printf "%s", $2}')/wasms/$1" | awk '{print "0x" $1}'
    }
    wasm_matches() {
        echo "Comparing $* ..."
        [[ "$(installed_wasm_hash "$1")" == "$(downloaded_wasm_hash "$2")" ]] || {
                echo "ERROR:  There is a wasm hash mismatch between $1 and $2"
                echo "ERROR:  $(installed_wasm_hash "$1") != $(downloaded_wasm_hash "$2")"
                exit 1
        }>&2
    }
    wasm_matches nns-registry registry-canister.wasm
    wasm_matches nns-governance governance-canister_test.wasm
    wasm_matches nns-ledger ledger-canister.wasm
    wasm_matches nns-root root-canister.wasm
    wasm_matches nns-cycles-minting cycles-minting-canister.wasm
    wasm_matches nns-lifeline lifeline_canister.wasm
    wasm_matches nns-genesis-token genesis-token-canister.wasm
    wasm_matches nns-sns-wasm sns-wasm-canister.wasm
    wasm_matches internet_identity internet_identity_dev.wasm
    wasm_matches nns-dapp nns-dapp_test.wasm

    echo "   Accounts should have funds..."
    account_has_funds() {
        run dfx ledger balance "$1"
        assert_success
        assert_output "1000000000.00000000 ICP"
    }
    SECP256K1_ACCOUNT_ID="2b8fbde99de881f695f279d2a892b1137bfe81a42d7694e064b1be58701e1138"
    ED25519_ACCOUNT_ID="5b315d2f6702cb3a27d826161797d7b2c2e131cd312aece51d4d5574d1247087"
    account_has_funds "$SECP256K1_ACCOUNT_ID"
    account_has_funds "$ED25519_ACCOUNT_ID"

    echo "    The Internet Identity and NNS dapp should load"
    curl "http://qhbym-qaaaa-aaaaa-aaafq-cai.localhost:8080" | grep "<title>Internet Identity</title>"
    curl "http://qsgjb-riaaa-aaaaa-aaaga-cai.localhost:8080" | gzip -d | grep "<title>NNS Dapp</title>"

    echo "    The secp256k1 account can be controlled from the command line"
    install_asset nns
    dfx identity import --force --disable-encryption ident-1 ident-1/identity.pem
    run dfx ledger account-id --identity ident-1
    assert_success
    assert_output "$SECP256K1_ACCOUNT_ID"

    echo "    The registry canister should be initialized"
    run dfx canister call rwlgt-iiaaa-aaaaa-aaaaa-cai get_subnet_for_canister '(record {"principal"=opt principal"rwlgt-iiaaa-aaaaa-aaaaa-cai"})'
    assert_success
    assert_output --partial "Ok = record"
    assert_output --partial "subnet_id = opt principal"
    run dfx canister call rwlgt-iiaaa-aaaaa-aaaaa-cai get_subnet_for_canister '(record {"principal"=opt principal"aaaaa-aa"})'
    assert_success
    assert_output --partial "Err = \"Invalid canister ID.\""

    sleep 10 # In slow CI the last upgrade proposal has not finished executing yet. Need to give a little spare time to restart all canisters
    run dfx --identity ident-1 ledger transfer 4b37224c5ed36e8a28ae39af482f5f858104f0a2285d100e67cf029ff07d948e --amount 10 --memo 1414416717
    assert_success
    run dfx --identity ident-1 canister call rkp4c-7iaaa-aaaaa-aaaca-cai notify_mint_cycles '(record { block_index = 5 : nat64; })'
    # If cycles ledger is configured correctly, then notify_mint_cycles will try to call the cycles ledger (and fail because the canister is not even created).
    # If it is not configured correctly, then this will complain about the cycles ledger canister id not being configured.
    assert_output --partial "Canister um5iw-rqaaa-aaaaq-qaaba-cai not found"
}

@test "dfx nns install with a canister type defined by another extension" {
    install_shared_asset subnet_type/shared_network_settings/system
    dfx_start_for_nns_install

    CACHE_DIR=$(dfx cache show)
    mkdir -p "$CACHE_DIR"/extensions/embera
    cat > "$CACHE_DIR"/extensions/embera/extension.json <<EOF
    {
      "name": "embera",
      "version": "0.1.0",
      "homepage": "https://github.com/dfinity/dfx-extensions",
      "authors": "DFINITY",
      "summary": "Test extension for e2e purposes.",
      "categories": [],
      "keywords": [],
      "canister_type": {
       "defaults": {
         "type": "custom",
         "wasm": ".embera/{{canister_name}}/{{canister_name}}.wasm"
       }
      }
    }
EOF
    cat > dfx.json <<EOF
    {
      "canisters": {
        "c1": {
          "type": "embera",
          "candid": "main.did",
          "main": "main-file.embera"
        }
      }
    }
EOF

    dfx nns install
}
