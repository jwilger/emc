#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

output=${GITHUB_OUTPUT:?GITHUB_OUTPUT must be set}
event_revision=$(git rev-parse HEAD)
trunk_revision=$(git rev-parse origin/main)

baseline_revision=$(git describe --tags --abbrev=0 "$trunk_revision" 2>/dev/null \
  || git rev-list --max-parents=0 "$trunk_revision")
release_revision=$(git log --format='%H%x09%s' \
  "${baseline_revision}..${trunk_revision}" \
  | awk -F '\t' '$2 ~ /^chore\(release\): v[0-9]/ { print $1; exit }')

package_version() {
  git show "$1:Cargo.toml" \
    | awk '
      /^\[package\]$/ { in_package = 1; next }
      /^\[/ { in_package = 0 }
      in_package && /^version[[:space:]]*=/ {
        sub(/^[^"]*"/, "")
        sub(/".*/, "")
        print
        exit
      }
    '
}

if [ -n "$release_revision" ]; then
  release_version=$(package_version "$release_revision")
  trunk_version=$(package_version "$trunk_revision")

  if [ -n "$release_version" ] && [ "$release_version" = "$trunk_version" ]; then
    git checkout -B release-plz-pending "$release_revision"
    git branch --set-upstream-to=origin/main release-plz-pending
    printf 'current=true\nrecovery=true\n' >>"$output"
    exit 0
  fi
fi

if [ "$event_revision" = "$trunk_revision" ]; then
  printf 'current=true\nrecovery=false\n' >>"$output"
  exit 0
fi

echo 'A newer non-release trunk revision exists; its CI run will release it instead.'
printf 'current=false\nrecovery=false\n' >>"$output"
