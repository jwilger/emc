#!/usr/bin/env bats
# Copyright 2026 John Wilger

setup() {
  repository="$BATS_TEST_TMPDIR/repository"
  output_file="$BATS_TEST_TMPDIR/github-output"
  git init --initial-branch=main "$repository"
  git -C "$repository" config user.name 'EMC test'
  git -C "$repository" config user.email 'emc@example.test'
  git -C "$repository" config commit.gpgsign false
  git -C "$repository" remote add origin "$repository"
  printf 'seed\n' >"$repository/README.md"
  cat >"$repository/Cargo.toml" <<'TOML'
[package]
name = "emc"
version = "0.1.12"
TOML
  git -C "$repository" add README.md Cargo.toml
  git -C "$repository" commit -m 'test: seed trunk'
  event_revision="$(git -C "$repository" rev-parse HEAD)"
}

create_release_commit() {
  printf 'release\n' >>"$repository/README.md"
  sed -i 's/version = "0.1.12"/version = "0.1.13"/' "$repository/Cargo.toml"
  git -C "$repository" add README.md Cargo.toml
  git -C "$repository" commit -m 'chore(release): v0.1.13'
  release_revision="$(git -C "$repository" rev-parse HEAD)"
  git -C "$repository" update-ref refs/remotes/origin/main "$release_revision"
  git -C "$repository" checkout --detach "$event_revision"
}

@test "a rerun resumes from the release-only child of its event revision" {
  create_release_commit

  run env GITHUB_OUTPUT="$output_file" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/select-release-trunk.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'current=true' "$output_file"
  grep -Fqx 'recovery=true' "$output_file"
  [ "$(git -C "$repository" rev-parse HEAD)" = "$release_revision" ]
  [ "$(git -C "$repository" symbolic-ref --short HEAD)" = 'release-plz-pending' ]
  [ "$(git -C "$repository" rev-parse --abbrev-ref '@{upstream}')" = 'origin/main' ]
}

@test "the current trunk revision is retained" {
  git -C "$repository" update-ref refs/remotes/origin/main "$event_revision"

  run env GITHUB_OUTPUT="$output_file" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/select-release-trunk.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'current=true' "$output_file"
  grep -Fqx 'recovery=false' "$output_file"
  [ "$(git -C "$repository" rev-parse HEAD)" = "$event_revision" ]
}

@test "a newer non-release trunk revision is rejected" {
  printf 'new work\n' >>"$repository/README.md"
  git -C "$repository" add README.md
  git -C "$repository" commit -m 'fix: newer trunk work'
  newer_revision="$(git -C "$repository" rev-parse HEAD)"
  git -C "$repository" update-ref refs/remotes/origin/main "$newer_revision"
  git -C "$repository" checkout --detach "$event_revision"

  run env GITHUB_OUTPUT="$output_file" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/select-release-trunk.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'current=false' "$output_file"
  [ "$(git -C "$repository" rev-parse HEAD)" = "$event_revision" ]
}

@test "an untagged release is resumed after newer non-release trunk commits" {
  create_release_commit
  git -C "$repository" checkout --detach "$release_revision"
  printf 'pipeline fix\n' >>"$repository/README.md"
  git -C "$repository" add README.md
  git -C "$repository" commit -m 'fix: pipeline follow-up'
  newer_revision="$(git -C "$repository" rev-parse HEAD)"
  git -C "$repository" update-ref refs/remotes/origin/main "$newer_revision"

  run env GITHUB_OUTPUT="$output_file" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/select-release-trunk.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'current=true' "$output_file"
  grep -Fqx 'recovery=true' "$output_file"
  [ "$(git -C "$repository" rev-parse HEAD)" = "$release_revision" ]
}

@test "the latest compatible generated release is resumed" {
  create_release_commit
  git -C "$repository" checkout --detach "$release_revision"
  printf 'refreshed changelog\n' >>"$repository/README.md"
  git -C "$repository" add README.md
  git -C "$repository" commit -m 'chore(release): v0.1.13'
  latest_release_revision="$(git -C "$repository" rev-parse HEAD)"
  git -C "$repository" update-ref refs/remotes/origin/main "$latest_release_revision"
  git -C "$repository" checkout --detach "$event_revision"

  run env GITHUB_OUTPUT="$output_file" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/select-release-trunk.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'recovery=true' "$output_file"
  [ "$(git -C "$repository" rev-parse HEAD)" = "$latest_release_revision" ]
}

@test "a reverted pending release leaves current trunk selected for recalculation" {
  create_release_commit
  git -C "$repository" checkout --detach "$release_revision"
  sed -i 's/version = "0.1.13"/version = "0.1.12"/' "$repository/Cargo.toml"
  git -C "$repository" add Cargo.toml
  git -C "$repository" commit -m 'revert: failed release version bump'
  reverted_revision="$(git -C "$repository" rev-parse HEAD)"
  git -C "$repository" update-ref refs/remotes/origin/main "$reverted_revision"

  run env GITHUB_OUTPUT="$output_file" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/select-release-trunk.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'current=true' "$output_file"
  grep -Fqx 'recovery=false' "$output_file"
  [ "$(git -C "$repository" rev-parse HEAD)" = "$reverted_revision" ]
}
