#!/usr/bin/env bats
# Copyright 2026 John Wilger

setup() {
  mock_bin="$BATS_TEST_TMPDIR/bin"
  command_log="$BATS_TEST_TMPDIR/commands.log"
  mkdir -p "$mock_bin"

  cat >"$mock_bin/git" <<'SCRIPT'
#!/bin/sh
printf 'git %s\n' "$*" >>"$COMMAND_LOG"
case "$1" in
  ls-remote)
    [ "${REMOTE_TAG_EXISTS:-false}" = true ]
    ;;
  tag|push)
    exit 0
    ;;
  rev-parse)
    printf '%s\n' release-commit
    ;;
  *)
    echo "unexpected git command: $*" >&2
    exit 1
    ;;
esac
SCRIPT

  cat >"$mock_bin/gh" <<'SCRIPT'
#!/bin/sh
printf 'gh %s\n' "$*" >>"$COMMAND_LOG"
case "$1 $2" in
  'release view')
    [ "${GITHUB_RELEASE_EXISTS:-false}" = true ]
    ;;
  'release create')
    exit 0
    ;;
  *)
    echo "unexpected gh command: $*" >&2
    exit 1
    ;;
esac
SCRIPT
  chmod +x "$mock_bin/git" "$mock_bin/gh"
}

@test "missing tag and release are completed after publication" {
  run env PATH="$mock_bin:$PATH" COMMAND_LOG="$command_log" \
    "$BATS_TEST_DIRNAME/../scripts/complete-published-release.sh" 0.1.13

  [ "$status" -eq 0 ]
  run cat "$command_log"
  [ "$output" = $'git ls-remote --exit-code --tags origin refs/tags/v0.1.13\ngit tag -s v0.1.13 -m v0.1.13 HEAD\ngit push origin refs/tags/v0.1.13\ngh release view v0.1.13\ngit rev-parse v0.1.13^{}\ngh release create v0.1.13 --target release-commit --title v0.1.13 --generate-notes' ]
}

@test "an existing tag and release are left untouched on rerun" {
  run env PATH="$mock_bin:$PATH" COMMAND_LOG="$command_log" \
    REMOTE_TAG_EXISTS=true GITHUB_RELEASE_EXISTS=true \
    "$BATS_TEST_DIRNAME/../scripts/complete-published-release.sh" 0.1.13

  [ "$status" -eq 0 ]
  run cat "$command_log"
  [ "$output" = $'git ls-remote --exit-code --tags origin refs/tags/v0.1.13\ngh release view v0.1.13' ]
}

@test "an existing tag with a missing release creates only the release" {
  run env PATH="$mock_bin:$PATH" COMMAND_LOG="$command_log" \
    REMOTE_TAG_EXISTS=true \
    "$BATS_TEST_DIRNAME/../scripts/complete-published-release.sh" 0.1.13

  [ "$status" -eq 0 ]
  run cat "$command_log"
  [ "$output" = $'git ls-remote --exit-code --tags origin refs/tags/v0.1.13\ngh release view v0.1.13\ngit rev-parse v0.1.13^{}\ngh release create v0.1.13 --target release-commit --title v0.1.13 --generate-notes' ]
}
