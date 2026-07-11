#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

output=${GITHUB_OUTPUT:?GITHUB_OUTPUT must be set}
event_revision=$(git rev-parse HEAD)
trunk_revision=$(git rev-parse origin/main)

baseline_revision=$(git describe --tags --abbrev=0 "$trunk_revision" 2>/dev/null \
  || git rev-list --max-parents=0 "$trunk_revision")
release_revision=$(git log --reverse --format='%H%x09%s' \
  "${baseline_revision}..${trunk_revision}" \
  | awk -F '\t' '$2 ~ /^chore\(release\): v[0-9]/ { print $1; exit }')

if [ -n "$release_revision" ]; then
  git checkout --detach "$release_revision"
  printf 'current=true\n' >>"$output"
  exit 0
fi

if [ "$event_revision" = "$trunk_revision" ]; then
  printf 'current=true\n' >>"$output"
  exit 0
fi

echo 'A newer non-release trunk revision exists; its CI run will release it instead.'
printf 'current=false\n' >>"$output"
