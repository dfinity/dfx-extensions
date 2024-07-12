#!/usr/bin/env bash
#
# Tested with dfx 0.20.1

set -euo pipefail

# This ID corresponds to TEST_NEURON_1
NEURON_ID="449479075714955186"

dfx start --clean --background

dfx identity use default
cargo run --bin nns install --dfx-cache-path="$(dfx cache show)"

# Ensure we have a powerful neuron
cat <<EOF >ident-1.pem
-----BEGIN EC PRIVATE KEY-----
MHQCAQEEICJxApEbuZznKFpV+VKACRK30i6+7u5Z13/DOl18cIC+oAcGBSuBBAAK
oUQDQgAEPas6Iag4TUx+Uop+3NhE6s3FlayFtbwdhRVjvOar0kPTfE/N8N6btRnd
74ly5xXEBNSXiENyxhEuzOZrIWMCNQ==
-----END EC PRIVATE KEY-----
EOF
dfx identity import --force --storage-mode=plaintext ident-1 ident-1.pem
dfx identity use ident-1
PRINCIPAL_ID=$(dfx identity get-principal)

# Hack
sleep 15s

# Top-up the SNS-W canister
dfx ledger fabricate-cycles --canister qaa6y-5yaaa-aaaaa-aaafa-cai --t 2345

dfx canister call "rrkah-fqaaa-aaaaa-aaaaq-cai" update_neuron '(
  record {
    id = opt record { id = '${NEURON_ID}' : nat64 };
    staked_maturity_e8s_equivalent = null;
    controller = opt principal "'${PRINCIPAL_ID}'";
    recent_ballots = vec {};
    kyc_verified = true;
    neuron_type = null;
    not_for_profit = false;
    maturity_e8s_equivalent = 1_000_000 : nat64;
    cached_neuron_stake_e8s = 1_000_000_000_000_000 : nat64;
    created_timestamp_seconds = 123 : nat64;
    auto_stake_maturity = opt true;
    aging_since_timestamp_seconds = 456 : nat64;
    hot_keys = vec {};
    account = blob "3\8fZ\9fn\af]\a9\17\be\ea\14yA\f3\b3\00\16\af[\ae\1cq\c0\a0\dd\1d?\d8\e7\a96";
    joined_community_fund_timestamp_seconds = opt (1 : nat64);
    dissolve_state = opt variant {
        DissolveDelaySeconds = 252_460_800 : nat64
    };
    followees = vec {};
    neuron_fees_e8s = 0 : nat64;
    transfer = null;
    known_neuron_data = null;
    spawn_at_timestamp_seconds = null;
  },
)'

curl https://raw.githubusercontent.com/dfinity/sns-testing/main/example_sns_init.yaml \
    | sed "s/YOUR_PRINCIPAL_ID/${PRINCIPAL_ID}/" \
    | sed "s/- YOUR_CANISTER_ID//" > sns_init.yaml

touch logo.png

cargo run --bin sns propose --neuron-id "${NEURON_ID}" sns_init.yaml

# Check that the CreateServiceNervousSystem propsoal was executed
PROPOSAL_DATA=$(dfx canister \
  call "rrkah-fqaaa-aaaaa-aaaaq-cai" \
  list_proposals '(
    record {
    include_reward_status = vec {};
    before_proposal = null;
    limit = 1;
    exclude_topic = vec {};
    include_status = vec {};
  }
)' | idl2json)

while [ "$(echo "${PROPOSAL_DATA}" | jq -r '.proposal_info[0].executed_timestamp_seconds')" == "0" ]
do
  FAILURE_REASON=$(echo "${PROPOSAL_DATA}" | jq -c -r '.proposal_info[0].failure_reason?')
  if [ "$FAILURE_REASON" = "null" ]; then
    printf "."
    sleep 1
  else
    echo "CreateServiceNervousSystem proposal FAILED: ${FAILURE_REASON}"
    dfx stop
    exit 1
  fi
done

echo "CreateServiceNervousSystem proposal SUCCEEDED!"
echo "Run `dfx stop` when you're done testing."
