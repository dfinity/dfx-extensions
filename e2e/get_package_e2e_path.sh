#!/usr/bin/env sh

PACKAGE="sns" # TODO: replace with $1
PACKAGE=$(cargo metadata --format-version=1 | jq -r '.workspace_members[]' | rg $PACKAGE)
GIT_ROOT_DIR=$(git rev-parse --show-toplevel)
# trim space at the end, and push `/e2e` at the end
PACKAGE=${PACKAGE%)}/e2e
# trim everything from beginning up until the relative repo path
PACKAGE=${PACKAGE#*"$GIT_ROOT_DIR"}
echo "$PACKAGE"
