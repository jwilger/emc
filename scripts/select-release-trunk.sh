#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

output=${GITHUB_OUTPUT:?GITHUB_OUTPUT must be set}
event_revision=$(git rev-parse HEAD)
trunk_revision=$(git rev-parse origin/main)

if [ "$event_revision" = "$trunk_revision" ]; then
  printf 'current=true\n' >>"$output"
  exit 0
fi

if [ "$(git rev-parse "${trunk_revision}^")" = "$event_revision" ] \
  && git log -1 --format=%s "$trunk_revision" | grep -Eq '^chore\(release\): v[0-9]'; then
  git checkout --detach "$trunk_revision"
  printf 'current=true\n' >>"$output"
  exit 0
fi

echo 'A newer non-release trunk revision exists; its CI run will release it instead.'
printf 'current=false\n' >>"$output"
