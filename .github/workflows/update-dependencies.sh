#!/usr/bin/env bash

# Usage: .github/workflows/update-dependencies.sh EXTENSION NEW_VERSION
# Run from project root

EXTENSION=$1
NEW_VERSION=$2

if [ -z "$EXTENSION" ] || [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 EXTENSION NEW_VERSION"
  exit 1
fi

EXTENSION_JSON="extensions/$EXTENSION/extension.json"
DEPENDENCIES_JSON="extensions/$EXTENSION/dependencies.json"

# Copy 'dependencies' field from extension.json to dependencies.json for the new version
jq --slurpfile extension "$EXTENSION_JSON" --arg version "$NEW_VERSION" '
  .[$version] = (reduce ($extension[0].dependencies | to_entries[]) as $dep (
    {}; .[$dep.key] = (if $dep.value | type == "string" # normalize short-form version strings
                       then {"version": $dep.value}     # to object with "version" key,
                       else $dep.value                  # leave others as-is
                       end)
  ))
  | {($version): .[$version]} + .                       # add new version as first entry
' "$DEPENDENCIES_JSON" | sponge "$DEPENDENCIES_JSON"
