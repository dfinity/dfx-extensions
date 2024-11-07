set -e

load "$GIT_ROOT_DIR"/e2e/bats-support/load
load "$GIT_ROOT_DIR"/e2e/bats-assert/load

# Takes a name of the asset folder, and copy those files to the current project.
install_asset() {
    ASSET_ROOT="$(dirname "$BATS_TEST_FILENAME")"/../assets/$1
    cp -R "$ASSET_ROOT"/* .

    # shellcheck source=/dev/null
    if [ -f ./patch.bash ]; then source ./patch.bash; fi
    if [ -f ./Cargo.toml ]; then cargo update; fi
}

install_shared_asset() {
    mkdir -p "$(dirname "$E2E_NETWORKS_JSON")"

    ASSET_ROOT="$(dirname "$BATS_TEST_FILENAME")"/../assets/$1
    cp -R "$ASSET_ROOT"/* "$(dirname "$E2E_NETWORKS_JSON")"
}

dfx_extension_install_manually() (
    local extension_name="$1"
    extensions_dir="$(dfx cache show)/extensions"
    mkdir -p "$extensions_dir"
    cp -R "$PREBUILT_EXTENSIONS_DIR/$extension_name" "$extensions_dir/$extension_name"
)

standard_setup() {
    # We want to work from a temporary directory, different for every test.
    x=$(mktemp -d -t dfx-e2e-XXXXXXXX)
    export E2E_TEMP_DIR="$x"

    cache_root="${E2E_CACHE_ROOT:-"$HOME/.e2e-cache-root"}"

    if [ "$(uname)" == "Darwin" ]; then
        project_relative_path="Library/Application Support/org.dfinity.dfx"
    elif [ "$(uname)" == "Linux" ]; then
        project_relative_path=".local/share/dfx"
    fi

    mkdir "$x/working-dir"
    mkdir -p "$cache_root"
    mkdir "$x/config-root"
    mkdir "$x/home-dir"

    # we need to configure dfxvm in the isolated home directory
    default_dfx_version="$(dfxvm default)"
    # don't re-download dfx for every test
    mkdir -p "$x/home-dir/$project_relative_path"
    ln -s "$HOME/$project_relative_path/versions" "$x/home-dir/$project_relative_path/versions"

    cd "$x/working-dir" || exit

    export HOME="$x/home-dir"
    export DFX_CACHE_ROOT="$cache_root"
    export DFX_CONFIG_ROOT="$x/config-root"
    export RUST_BACKTRACE=1
    export MOCK_KEYRING_LOCATION="$HOME/mock_keyring.json"

    export E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY="$HOME/$project_relative_path/network/local"
    export E2E_NETWORKS_JSON="$DFX_CONFIG_ROOT/.config/dfx/networks.json"

    dfxvm default "$default_dfx_version"
    dfx cache install
}

standard_teardown() {
    rm -rf "$E2E_TEMP_DIR" || rm -rf "$E2E_TEMP_DIR"
}

dfx_new_frontend() {
    local project_name=${1:-e2e_project}
    dfx new "${project_name}" --frontend simple-assets
    test -d "${project_name}"
    test -f "${project_name}"/dfx.json
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

dfx_new() {
    local project_name=${1:-e2e_project}
    dfx new "${project_name}" --no-frontend
    test -d "${project_name}"
    test -f "${project_name}/dfx.json"
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

dfx_new_rust() {
    local project_name=${1:-e2e_project}
    rustup default stable
    rustup target add wasm32-unknown-unknown
    dfx new "${project_name}" --type=rust --no-frontend
    test -d "${project_name}"
    test -f "${project_name}/dfx.json"
    test -f "${project_name}/Cargo.toml"
    test -f "${project_name}/Cargo.lock"
    cd "${project_name}"

    echo PWD: "$(pwd)" >&2
}

dfx_patchelf() {
    # Don't run this function during github actions
    [ "$GITHUB_ACTIONS" ] && return 0

    # Only run this function on Linux
    (uname -a | grep Linux) || return 0

    local CACHE_DIR LD_LINUX_SO BINARY IS_STATIC USE_LIB64

    echo dfx = "$(which dfx)"
    CACHE_DIR="$(dfx cache show)"

    # Both ldd and iconv are providedin glibc.bin package
    LD_LINUX_SO=$(ldd "$(which iconv)"|grep ld-linux-x86|cut -d' ' -f3)
    for binary in ic-starter icx-proxy replica; do
        BINARY="${CACHE_DIR}/${binary}"
        test -f "$BINARY" || continue
        IS_STATIC=$(ldd "${BINARY}" | grep 'not a dynamic executable')
        USE_LIB64=$(ldd "${BINARY}" | grep '/lib64/ld-linux-x86-64.so.2')
        chmod +rw "${BINARY}"
        test -n "$IS_STATIC" || test -z "$USE_LIB64" || patchelf --set-interpreter "${LD_LINUX_SO}" "${BINARY}"
    done
}

determine_network_directory() {
    # not perfect: dfx.json can actually exist in a parent
    if [ -f dfx.json ] && [ "$(jq .networks.local dfx.json)" != "null" ]; then
        echo "found dfx.json with local network in $(pwd)"
        data_dir="$(pwd)/.dfx/network/local"
        wallets_json="$(pwd)/.dfx/local/wallets.json"
        dfx_json="$(pwd)/dfx.json"
        export E2E_NETWORK_DATA_DIRECTORY="$data_dir"
        export E2E_NETWORK_WALLETS_JSON="$wallets_json"
        export E2E_ROUTE_NETWORKS_JSON="$dfx_json"
    else
        echo "no dfx.json"
        export E2E_NETWORK_DATA_DIRECTORY="$E2E_SHARED_LOCAL_NETWORK_DATA_DIRECTORY"
        export E2E_NETWORK_WALLETS_JSON="$E2E_NETWORK_DATA_DIRECTORY/wallets.json"
        export E2E_ROUTE_NETWORKS_JSON="$E2E_NETWORKS_JSON"
    fi
}

# Tries to start dfx on the default port, repeating until it succeeds or times out.
#
# Motivation: dfx nns install works only on port 8080, as URLs are compiled into the wasms.  This means that multiple
# tests MAY compete for the same port.
# - It may be possible in future for the wasms to detect their own URL and recompute signatures accordingly,
#   however until such a time, we have this restriction.
# - It may also be that ic-nns-install, if used on a non-standard port, installs only the core canisters not the UI.
# - However until we have implemented good solutions, all tests on ic-nns-install must run on port 8080.
dfx_start_for_nns_install() {
    if [ "$USE_POCKET_IC" ]
    then
        DFX_START="dfx start --pocketic"
    else
        DFX_START="dfx start"
    fi

    # TODO: When nns-dapp supports dynamic ports, this wait can be removed.
    timeout 300 sh -c \
        "until $DFX_START --clean --background --host 127.0.0.1:8080 --verbose ; do echo waiting for port 8080 to become free; sleep 3; done" \
        || (echo "could not connect to replica on port 8080" && exit 1)
    # TODO: figure out how to plug bats' "run" into above statement,
    #       so that below asserts will work as expected
    # assert_success
    # assert_output --partial "subnet type: System"
    # assert_output --partial "bind address: 127.0.0.1:8080"
}

wait_until_replica_healthy() {
    echo "waiting for replica to become healthy"
    dfx ping --wait-healthy
    echo "replica became healthy"
}

dfx_bootstrap() {
    # This only works because we use the network by name
    #    (implicitly: --network local)
    # If we passed --network http://127.0.0.1:${replica_port}
    # we would get errors like this:
    #    "Cannot find canister ryjl3-tyaaa-aaaaa-aaaba-cai for network http___127_0_0_1_54084"
    dfx bootstrap --port 0 3>&- &
    export DFX_BOOTSTRAP_PID=$!

    timeout 5 sh -c \
        "until nc -z localhost \$(cat \"$E2E_NETWORK_DATA_DIRECTORY/webserver-port\"); do echo waiting for webserver; sleep 1; done" \
        || (echo "could not connect to webserver on port $(get_webserver_port)" && exit 1)

    wait_until_replica_healthy

    webserver_port=$(cat "$E2E_NETWORK_DATA_DIRECTORY/webserver-port")
    printf "Webserver Configured Port: %s\n", "${webserver_port}"
}

# Stop the `dfx replica` process that is running in the background.
stop_dfx_replica() {
    [ "$DFX_REPLICA_PID" ] && kill -TERM "$DFX_REPLICA_PID"
    unset DFX_REPLICA_PID
}

# Stop the `dfx bootstrap` process that is running in the background
stop_dfx_bootstrap() {
    [ "$DFX_BOOTSTRAP_PID" ] && kill -TERM "$DFX_BOOTSTRAP_PID"
    unset DFX_BOOTSTRAP_PID
}

# Stop the replica and verify it is very very stopped.
dfx_stop() {
    # to help tell if other icx-proxy processes are from this test:
    echo "pwd: $(pwd)"
    # A suspicion: "address already is use" errors are due to an extra icx-proxy process.
    echo "icx-proxy processes:"
    pgrep -l icx-proxy || echo "no ps/grep/icx-proxy output"

    dfx stop
    local dfx_root=.dfx/
    rm -rf $dfx_root

    # Verify that processes are killed.
    assert_no_dfx_start_or_replica_processes
}

dfx_set_wallet() {
  export WALLET_CANISTER_ID
  WALLET_CANISTER_ID=$(dfx identity get-wallet)
  assert_command dfx identity set-wallet "${WALLET_CANISTER_ID}" --force --network actuallylocal
  assert_match 'Wallet set successfully.'
}

setup_actuallylocal_project_network() {
    webserver_port=$(get_webserver_port)
    # [ ! -f "$E2E_ROUTE_NETWORKS_JSON" ] && echo "{}" >"$E2E_ROUTE_NETWORKS_JSON"
    jq '.networks.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' dfx.json | sponge dfx.json
}

setup_actuallylocal_shared_network() {
    webserver_port=$(get_webserver_port)
    [ ! -f "$E2E_NETWORKS_JSON" ] && echo "{}" >"$E2E_NETWORKS_JSON"
    jq '.actuallylocal.providers=["http://127.0.0.1:'"$webserver_port"'"]' "$E2E_NETWORKS_JSON" | sponge "$E2E_NETWORKS_JSON"
}

use_wallet_wasm() {
    # shellcheck disable=SC2154
    export DFX_WALLET_WASM="${archive}/wallet/$1/wallet.wasm"
}

use_asset_wasm() {
    # shellcheck disable=SC2154
    export DFX_ASSETS_WASM="${archive}/frontend/$1/assetstorage.wasm.gz"
}

wallet_sha() {
    shasum -a 256 "${archive}/wallet/$1/wallet.wasm" | awk '{ print $1 }'
}

use_default_wallet_wasm() {
    unset DFX_WALLET_WASM
}

use_default_asset_wasm() {
    unset DFX_ASSETS_WASM
}

get_webserver_port() {
  dfx info webserver-port
}
overwrite_webserver_port() {
  echo "$1" >"$E2E_NETWORK_DATA_DIRECTORY/webserver-port"
}

get_replica_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-pid"
}

get_replica_port() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/replica-configuration/replica-1.port"
}

get_btc_adapter_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-btc-adapter-pid"
}

get_canister_http_adapter_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/ic-canister-http-adapter-pid"
}

get_icx_proxy_pid() {
  cat "$E2E_NETWORK_DATA_DIRECTORY/icx-proxy-pid"
}

create_networks_json() {
  mkdir -p "$(dirname "$E2E_NETWORKS_JSON")"
  [ ! -f "$E2E_NETWORKS_JSON" ] && echo "{}" >"$E2E_NETWORKS_JSON"
}

define_project_network() {
    jq .networks.local.bind=\"127.0.0.1:8000\" dfx.json | sponge dfx.json
}

use_test_specific_cache_root() {
    # Use this when a test depends on the initial state of the cache being empty,
    # or if the test corrupts the cache in some way.
    # The effect is to ignore the E2E_CACHE_ROOT environment variable, if set.
    export DFX_CACHE_ROOT="$E2E_TEMP_DIR/cache-root"
    mkdir -p "$DFX_CACHE_ROOT"
}

start_webserver() {
    local port script_dir
    script_dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
    port=$(python3 "$script_dir/get_ephemeral_port.py")
    export E2E_WEB_SERVER_PORT="$port"

    python3 -m http.server "$E2E_WEB_SERVER_PORT" "$@" &
    export E2E_WEB_SERVER_PID=$!

    while ! nc -z localhost "$E2E_WEB_SERVER_PORT"; do
      sleep 1
    done
}

stop_webserver() {
    if [ "$E2E_WEB_SERVER_PID" ]; then
        kill "$E2E_WEB_SERVER_PID"
    fi
}

# Asserts that the contents of two files are equal.
# Arguments:
#    $1 - The name of the file containing the expected value.
#    $2 - The name of the file containing the actual value.
assert_files_eq() {
    expected="$(cat "$1")"
    actual="$(cat "$2")"

    if [[ ! "$actual" == "$expected" ]]; then
        diff "$1" "$2" \
            | batslib_decorate "contents of $1 do not match contents of $2" \
            | fail
    fi
}

# Asserts that `dfx start` and `replica` are no longer running
assert_no_dfx_start_or_replica_processes() {
    ! ( pgrep "dfx start" )
    if [ -e .dfx/replica-configuration/replica-pid ];
    then
      ! ( kill -0 "$(< .dfx/replica-configuration/replica-pid)" 2>/dev/null )
    fi
}
