#!/usr/bin/env bats
# Copyright 2026 John Wilger

setup() {
  project="$BATS_TEST_TMPDIR/project"
  mkdir -p "$project/.githooks"
  cp -R "$BATS_TEST_DIRNAME/../scripts/." "$project/scripts/"
  cp -R "$BATS_TEST_DIRNAME/../.githooks/." "$project/.githooks/"
  chmod +x "$project/scripts/"*.sh "$project/.githooks/"*
  git -C "$project" init --initial-branch=main
  git -C "$project" config user.name "EMC test"
  git -C "$project" config user.email "emc@example.test"
  git -C "$project" config commit.gpgsign false
  printf '.envrc\ntarget\n.worktrees/\n' >"$project/.gitignore"
  printf 'LOCAL_ONLY=1\n' >"$project/.envrc"
  printf 'seed\n' >"$project/README.md"
  git -C "$project" add .githooks .gitignore README.md scripts
  git -C "$project" commit -m 'test: seed repository'
  git -C "$project" config core.hooksPath "$project/.githooks"
  git init --bare "$BATS_TEST_TMPDIR/remote.git"
  git -C "$project" remote add origin "$BATS_TEST_TMPDIR/remote.git"
}

@test "worktree-create makes an isolated checkout that references the primary environment and target cache" {
  run just --justfile "$BATS_TEST_DIRNAME/../justfile" --working-directory "$project" worktree-create feature-one

  [ "$status" -eq 0 ]
  [ -d "$project/.worktrees/feature-one" ]
  [ -L "$project/.worktrees/feature-one/.envrc" ]
  [ "$(readlink "$project/.worktrees/feature-one/.envrc")" = "$project/.envrc" ]
  [ -L "$project/.worktrees/feature-one/target" ]
  [ "$(readlink "$project/.worktrees/feature-one/target")" = "$project/target" ]
}

@test "main checkout guard rejects commits while a linked worktree permits them" {
  run just --justfile "$BATS_TEST_DIRNAME/../justfile" --working-directory "$project" worktree-create feature-two
  [ "$status" -eq 0 ]

  printf 'main change\n' >>"$project/README.md"
  git -C "$project" add README.md
  run git -C "$project" commit -m 'test: blocked main commit'
  [ "$status" -ne 0 ]
  [[ "$output" == *"main checkout are blocked"* ]]

  run git -C "$project" push origin HEAD:refs/heads/main
  [ "$status" -ne 0 ]
  [[ "$output" == *"main checkout are blocked"* ]]

  worktree="$project/.worktrees/feature-two"
  printf 'worktree change\n' >"$worktree/feature.txt"
  git -C "$worktree" add feature.txt
  run git -C "$worktree" commit -m 'test: permitted worktree commit'
  [ "$status" -eq 0 ]

  run git -C "$worktree" push origin HEAD:refs/heads/worktree-feature-two
  [ "$status" -eq 0 ]
}

@test "worktree-remove removes the linked checkout without copying local environment files" {
  run just --justfile "$BATS_TEST_DIRNAME/../justfile" --working-directory "$project" worktree-create feature-three
  [ "$status" -eq 0 ]

  run just --justfile "$BATS_TEST_DIRNAME/../justfile" --working-directory "$project" worktree-remove feature-three

  [ "$status" -eq 0 ]
  [ ! -e "$project/.worktrees/feature-three" ]
}
