#!/usr/bin/env bats
# Copyright 2026 John Wilger

setup() {
  source_repository="$BATS_TEST_DIRNAME/.."
  repository="$BATS_TEST_TMPDIR/repository"
}

prepare_checkout() {
  git clone --quiet "$source_repository" "$repository"
  git -C "$repository" config user.name 'EMC release probe'
  git -C "$repository" config user.email 'release-probe@example.test'
  git -C "$repository" config commit.gpgsign false
}

run_update() {
  run sh -c 'cd "$1" && release-plz update --verbose --allow-dirty' sh "$repository"
}

package_version() {
  awk -F '"' '$1 == "version = " { print $2; exit }' "$repository/Cargo.toml"
}

next_minor_version() {
  IFS=. read -r major minor _ <<EOF
$(package_version)
EOF
  printf '%s.%s.0\n' "$major" "$((minor + 1))"
}

next_patch_version() {
  IFS=. read -r major minor patch <<EOF
$(package_version)
EOF
  printf '%s.%s.%s\n' "$major" "$minor" "$((patch + 1))"
}

@test "release-plz escalates a misleading patch change with a public API incompatibility" {
  prepare_checkout
  expected_version="$(next_minor_version)"
  sed -i '/    GuidanceCatalog, GuidanceTopic, VERSION, guidance_catalog, modeling_process_guide,/s/modeling_process_guide,//' "$repository/src/lib.rs"
  git -C "$repository" add src/lib.rs
  git -C "$repository" commit -m 'fix: clarify embedded guidance API'

  run_update

  [ "$status" -eq 0 ]
  [[ "$output" == *'Checking API compatibility with cargo-semver-checks...'* ]]
  [[ "$output" == *'API breaking changes'* ]]
  [ "$(package_version)" = "$expected_version" ]
}

@test "release-plz selects a minor version for an additive public API change" {
  prepare_checkout
  expected_version="$(next_minor_version)"
  printf '\n/// Release-versioning probe API.\n#[must_use]\npub const fn release_versioning_probe() -> bool {\n    true\n}\n' >>"$repository/src/lib.rs"
  git -C "$repository" add src/lib.rs
  git -C "$repository" commit -m 'feat: add a release versioning probe API'

  run_update

  [ "$status" -eq 0 ]
  [[ "$output" == *'Checking API compatibility with cargo-semver-checks...'* ]]
  [[ "$output" == *'API compatible changes'* ]]
  [ "$(package_version)" = "$expected_version" ]
}

@test "release-plz selects a patch version for an internal-only fix" {
  prepare_checkout
  expected_version="$(next_patch_version)"
  printf '\nconst RELEASE_VERSIONING_PROBE_INTERNAL_FIX: bool = true;\n' >>"$repository/src/lib.rs"
  git -C "$repository" add src/lib.rs
  git -C "$repository" commit -m 'fix: clarify release probe internals'

  run_update

  [ "$status" -eq 0 ]
  [[ "$output" == *'Checking API compatibility with cargo-semver-checks...'* ]]
  [[ "$output" == *'API compatible changes'* ]]
  [ "$(package_version)" = "$expected_version" ]
}
