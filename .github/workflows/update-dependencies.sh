#!/usr/bin/env bash

# Usage: ./update_dependencies.sh EXTENSION NEW_VERSION
# Run from project root

EXTENSION=$1
NEW_VERSION=$2

# ensure both parameters provided
if [ -z "$EXTENSION" ] || [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 EXTENSION NEW_VERSION"
  exit 1
fi

EXTENSION_JSON="extensions/$EXTENSION/extension.json"
DEPENDENCIES_JSON="extensions/$EXTENSION/dependencies.json"

jq --slurpfile extension "$EXTENSION_JSON" --arg version "$NEW_VERSION" '
  .[$version] = (reduce ($extension[0].dependencies | to_entries[]) as $dep (
    {}; .[$dep.key] = (if $dep.value | type == "string"
                       then {"version": $dep.value}
                       else $dep.value
                       end)
  ))
  | {($version): .[$version]} + .
' "$DEPENDENCIES_JSON" | sponge "$DEPENDENCIES_JSON"
