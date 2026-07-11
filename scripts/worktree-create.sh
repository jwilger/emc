#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

name=${1:?usage: worktree-create.sh <name>}
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
  printf '%s\n' 'worktrees: create linked worktrees from the primary checkout.' >&2
  exit 2
fi

destination="$root/.worktrees/$name"
branch="worktree/$name"
if [ -e "$destination" ] || git show-ref --verify --quiet "refs/heads/$branch"; then
  printf '%s\n' "worktrees: $name already exists." >&2
  exit 2
fi

mkdir -p "$root/.worktrees" "$root/target"
git config core.hooksPath "$root/.githooks"
git worktree add "$destination" -b "$branch" HEAD

if [ -e "$root/.envrc" ]; then
  ln -s "$root/.envrc" "$destination/.envrc"
fi
ln -s "$root/target" "$destination/target"
printf '%s\n' "created $destination"
