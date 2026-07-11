#!/usr/bin/env bats
# Copyright 2026 John Wilger

setup() {
  repository="$BATS_TEST_TMPDIR/repository"
  mock_bin="$BATS_TEST_TMPDIR/bin"
  command_log="$BATS_TEST_TMPDIR/release-plz.log"
  mkdir -p "$repository" "$mock_bin"
  git -C "$repository" init --initial-branch=main
  git -C "$repository" config user.name 'EMC test'
  git -C "$repository" config user.email 'emc@example.test'
  git -C "$repository" config commit.gpgsign false
  printf 'seed\n' >"$repository/README.md"
  git -C "$repository" add README.md
  git -C "$repository" commit -m 'test: seed release range'

  cat >"$mock_bin/release-plz" <<'SCRIPT'
#!/usr/bin/env sh
printf '%s\n' "$*" >>"$RELEASE_PLZ_LOG"
case "${RELEASE_PLZ_RESULT:-semver}" in
  semver)
    printf '%s\n' 'Checking API compatibility with cargo-semver-checks...'
    printf '%s\n' '(✓ API compatible changes)'
    ;;
  no-semver) printf '%s\n' 'updated changelog' ;;
  skipped)
    printf '%s\n' 'Checking API compatibility with cargo-semver-checks...'
    printf '%s\n' 'cargo-semver-checks: skipped'
    ;;
  semver-error)
    printf '%s\n' 'Checking API compatibility with cargo-semver-checks...'
    printf '%s\n' '(✓ API compatible changes)'
    printf '%s\n' 'error while running cargo-semver-checks on emc'
    ;;
  skip-before)
    printf '%s\n' 'Checking API compatibility with cargo-semver-checks...'
    printf '%s\n' '(✓ API compatible changes)'
    printf '%s\n' 'Skipping cargo-semver-checks because the baseline could not be loaded'
    ;;
  baseline-success)
    printf '%s\n' 'Checking API compatibility with cargo-semver-checks...'
    printf '%s\n' 'baseline successfully loaded'
    printf '%s\n' '(✓ API compatible changes)'
    ;;
  error) exit 1 ;;
esac
SCRIPT
  chmod +x "$mock_bin/release-plz"
}

@test "release update fails closed when release-plz omits cargo-semver-checks evidence" {
  run env RELEASE_PLZ_RESULT=no-semver RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -ne 0 ]
  grep -Fqx 'update --verbose' "$command_log"
}

@test "release update fails closed when cargo-semver-checks reports it was skipped" {
  run env RELEASE_PLZ_RESULT=skipped RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -ne 0 ]
  grep -Fqx 'update --verbose' "$command_log"
}

@test "release update fails closed when cargo-semver-checks reports an error after a result" {
  run env RELEASE_PLZ_RESULT=semver-error RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -ne 0 ]
  grep -Fqx 'update --verbose' "$command_log"
}

@test "release update fails closed when cargo-semver-checks cannot load its baseline" {
  run env RELEASE_PLZ_RESULT=skip-before RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -ne 0 ]
  grep -Fqx 'update --verbose' "$command_log"
}

@test "release update accepts successful cargo-semver-checks evidence" {
  run env RELEASE_PLZ_RESULT=semver RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'update --verbose' "$command_log"
}

@test "release update uses the supplied validated configuration" {
  config="$BATS_TEST_TMPDIR/release-plz.toml"
  : >"$config"

  run env RELEASE_PLZ_CONFIG="$config" RELEASE_PLZ_RESULT=semver \
    RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -eq 0 ]
  grep -Fqx "update --verbose --config $config" "$command_log"
}

@test "release update accepts a successfully loaded semver baseline" {
  run env RELEASE_PLZ_RESULT=baseline-success RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -eq 0 ]
  grep -Fqx 'update --verbose' "$command_log"
}

@test "release update preserves a failed release-plz diagnostic and refuses publication" {
  run env RELEASE_PLZ_RESULT=error RELEASE_PLZ_LOG="$command_log" PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-green-trunk-update.sh"

  [ "$status" -ne 0 ]
  [[ "$output" == *'release-plz update failed; refusing to publish'* ]]
  grep -Fqx 'update --verbose' "$command_log"
}
