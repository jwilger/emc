#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

name=${1:?usage: worktree-remove.sh <name>}
case "$name" in
  *[!A-Za-z0-9._-]* | '' | . | ..)
    printf '%s\n' "worktrees: invalid worktree name: $name" >&2
    exit 2
    ;;
esac

root=$(git rev-parse --show-toplevel)
git_dir=$(git rev-parse --path-format=absolute --git-dir)
common_dir=$(git rev-parse --path-format=absolute --git-common-dir)
if [ "$git_dir" != "$common_dir" ]; then
  printf '%s\n' 'worktrees: remove linked worktrees from the primary checkout.' >&2
  exit 2
fi

destination="$root/.worktrees/$name"
if [ ! -d "$destination" ]; then
  printf '%s\n' "worktrees: $name does not exist." >&2
  exit 2
fi

git worktree remove "$destination"
printf '%s\n' "removed $destination"
