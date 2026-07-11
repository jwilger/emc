#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

git_dir=$(git rev-parse --path-format=absolute --git-dir)
common_dir=$(git rev-parse --path-format=absolute --git-common-dir)

if [ "$git_dir" = "$common_dir" ]; then
  printf '%s\n' 'worktrees: changes from the main checkout are blocked — work in a linked worktree.' >&2
  exit 1
fi
