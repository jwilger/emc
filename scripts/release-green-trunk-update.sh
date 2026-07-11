#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

output=$(mktemp)
trap 'rm -f "$output"' 0

if [ -n "${RELEASE_PLZ_CONFIG:-}" ]; then
  update_release_plz() {
    release-plz update --verbose --config "$RELEASE_PLZ_CONFIG"
  }
else
  update_release_plz() {
    release-plz update --verbose
  }
fi

if ! update_release_plz >"$output" 2>&1; then
  cat "$output" >&2
  echo 'release-plz update failed; refusing to publish' >&2
  exit 1
fi

cat "$output"
if ! grep -Fq 'Checking API compatibility with cargo-semver-checks...' "$output"; then
  echo 'cargo-semver-checks evidence is missing; refusing to publish' >&2
  exit 1
fi

if ! grep -Eq 'API compatible changes|API breaking changes' "$output"; then
  echo 'cargo-semver-checks did not report a compatibility result; refusing to publish' >&2
  exit 1
fi

if grep -Eiq '(error|fail|skipp).*(cargo-semver-checks|semver check)|(cargo-semver-checks|semver check).*(skipp|error|fail)|(could not|cannot|failed to|unable to).*(load|retrieve).*(baseline)|(load|retrieve).*(baseline).*(could not|cannot|failed|unable)|baseline.*(could not|cannot|failed|unable|unavailable)|unavailable.*baseline' "$output"; then
  echo 'cargo-semver-checks reported an incomplete or failed check; refusing to publish' >&2
  exit 1
fi
