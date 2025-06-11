#!/usr/bin/env bash

set -e -o pipefail

export LMDB_H_PATH="$(brew --prefix lmdb)/include/lmdb.h"
export LMDB_NO_BUILD=true

build_manually() (
  local extension_name="$1"
  package_version=$(cargo metadata --format-version=1 | jq -r --arg name "$extension_name" '.packages[] | select(.name == $name) | .version')
  echo "package version for $extension_name: $package_version"
  cargo dist build --tag="$extension_name-v$package_version" # cargo-dist needs git tag only metadata-related stuff; it won't do git checkout, it will build from HEAD
  extension_dir="$PREBUILT_EXTENSIONS_DIR/$extension_name"
  arch_platform="$(get_arch_and_platform)"
  mkdir -p "${extension_dir}"
  tar xzf "target/distrib/$extension_name-$arch_platform.tar.gz" --strip-components 1 -C "$extension_dir"
)

get_arch_and_platform() {
  ARCH=$(uname -m)
  SYS=$(uname -s)

  if [[ "$ARCH" == "x86_64" ]]; then
    if [[ "$SYS" == "Darwin" ]]; then
      echo "$ARCH-apple-darwin"
    elif [[ "$SYS" == "Linux" ]]; then
      echo "$ARCH-unknown-linux-gnu"
    else
      echo "System not recognized"
    fi
  elif [[ "$ARCH" == "arm64" && "$SYS" == "Darwin" ]]; then
    echo "aarch64-apple-darwin"
  else
    echo "Architecture not recognized"
  fi
}

build_manually "$1"
