#!/usr/bin/env bats
# Copyright 2026 John Wilger

setup() {
  repository="$BATS_TEST_TMPDIR/repository"
  mock_bin="$BATS_TEST_TMPDIR/bin"
  gh_log="$BATS_TEST_TMPDIR/gh.log"
  mkdir -p "$repository" "$mock_bin"
  git -C "$repository" init --initial-branch=main
  git -C "$repository" config user.name "EMC test"
  git -C "$repository" config user.email "emc@example.test"
  git -C "$repository" config commit.gpgsign false
  printf 'seed\n' >"$repository/README.md"
  git -C "$repository" add README.md
  git -C "$repository" commit -m 'test: seed release range'
  release_base="$(git -C "$repository" rev-parse HEAD)"
cat >"$mock_bin/gh" <<'SCRIPT'
#!/usr/bin/env sh
printf '%s\n' "$*" >>"$GH_LOG"
if [ "${GH_API_FAIL:-}" = 1 ] && [ "$1" = api ]; then
  exit 1
fi
if [ "$1" = issue ] && [ "$2" = comment ]; then
  shift 2
  while [ "$#" -gt 0 ]; do
    if [ "$1" = --body ]; then
      printf '%s' "$2" >"$GH_COMMENT_BODY"
      break
    fi
    shift
  done
fi
case "$*" in
  api*) printf '%s\n' "${GH_COMMENTS:-}" ;;
  *'--jq .labels[].name'*) printf '%s\n' "${GH_LABELS:-status:accepted}" ;;
  *'--jq .state'*) printf '%s\n' "${GH_STATE:-OPEN}" ;;
esac
SCRIPT
  chmod +x "$mock_bin/gh"
}

commit_with_trailer() {
  trailer="$1"
  printf '%s\n' "$trailer" >>"$repository/README.md"
  git -C "$repository" add README.md
  git -C "$repository" commit -m "fix: synchronize published release" -m "$trailer"
}

@test "release updates an issue once without closing it when only an update trailer is present" {
  commit_with_trailer 'Release-Updates-Issue: 17'
  release_head="$(git -C "$repository" rev-parse HEAD)"

  comment_body="$BATS_TEST_TMPDIR/comment-body"
  run env GH_COMMENT_BODY="$comment_body" GH_LOG="$gh_log" GITHUB_REPOSITORY='jwilger/emc' PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-issue-sync.sh" "$release_base" "$release_head" \
    '0.1.13' 'https://github.com/jwilger/emc/releases/tag/v0.1.13'

  [ "$status" -eq 0 ]
  [ "$(grep -c '^issue comment 17 ' "$gh_log")" -eq 1 ]
  [ "$(sed -n '1p' "$comment_body")" = 'Released in 0.1.13: https://github.com/jwilger/emc/releases/tag/v0.1.13' ]
  [ -z "$(sed -n '2p' "$comment_body")" ]
  grep -Fqx '<!-- emc-release:0.1.13:issue-17 -->' "$comment_body"
  ! grep -q '^issue close 17 ' "$gh_log"
}

@test "release fails closed when publication evidence cannot be read" {
  commit_with_trailer 'Release-Updates-Issue: 19'
  release_head="$(git -C "$repository" rev-parse HEAD)"

  run env GH_API_FAIL=1 GH_LOG="$gh_log" GITHUB_REPOSITORY='jwilger/emc' PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-issue-sync.sh" "$release_base" "$release_head" \
    '0.1.13' 'https://github.com/jwilger/emc/releases/tag/v0.1.13'

  [ "$status" -ne 0 ]
  ! grep -q '^issue comment 19 ' "$gh_log"
}

@test "release closes a deduplicated issue after recording its publication evidence" {
  commit_with_trailer 'Release-Closes-Issue: 18'
  commit_with_trailer 'Release-Closes-Issue: 18'
  release_head="$(git -C "$repository" rev-parse HEAD)"

  run env GH_LOG="$gh_log" GITHUB_REPOSITORY='jwilger/emc' PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-issue-sync.sh" "$release_base" "$release_head" \
    '0.1.13' 'https://github.com/jwilger/emc/releases/tag/v0.1.13'

  [ "$status" -eq 0 ]
  [ "$(grep -c '^issue comment 18 ' "$gh_log")" -eq 1 ]
  [ "$(grep -c '^issue edit 18 ' "$gh_log")" -eq 1 ]
  [ "$(grep -c '^issue close 18 ' "$gh_log")" -eq 1 ]
}

@test "release removes the accepted label when a closing issue is already released" {
  commit_with_trailer 'Release-Closes-Issue: 20'
  release_head="$(git -C "$repository" rev-parse HEAD)"

  run env GH_LABELS="$(printf 'status:accepted\nstatus:released')" GH_LOG="$gh_log" GITHUB_REPOSITORY='jwilger/emc' PATH="$mock_bin:$PATH" \
    sh -c 'cd "$1" && shift && exec "$@"' sh "$repository" \
    "$BATS_TEST_DIRNAME/../scripts/release-issue-sync.sh" "$release_base" "$release_head" \
    '0.1.13' 'https://github.com/jwilger/emc/releases/tag/v0.1.13'

  [ "$status" -eq 0 ]
  [ "$(grep -c '^issue edit 20 ' "$gh_log")" -eq 1 ]
}

@test "release reruns without duplicate issue mutations after publication evidence is recorded" {
  commit_with_trailer 'Release-Closes-Issue: 21'
  release_head="$(git -C "$repository" rev-parse HEAD)"
  marker='<!-- emc-release:0.1.13:issue-21 -->'
  command=(sh -c 'cd "$1" && shift && exec "$@"' sh "$repository"
    "$BATS_TEST_DIRNAME/../scripts/release-issue-sync.sh" "$release_base" "$release_head"
    '0.1.13' 'https://github.com/jwilger/emc/releases/tag/v0.1.13')

  run env GH_LOG="$gh_log" GITHUB_REPOSITORY='jwilger/emc' PATH="$mock_bin:$PATH" "${command[@]}"

  [ "$status" -eq 0 ]
  run env GH_COMMENTS="$marker" GH_LABELS='status:released' GH_STATE=CLOSED \
    GH_LOG="$gh_log" GITHUB_REPOSITORY='jwilger/emc' PATH="$mock_bin:$PATH" "${command[@]}"

  [ "$status" -eq 0 ]
  [ "$(grep -c '^issue comment 21 ' "$gh_log")" -eq 1 ]
  [ "$(grep -c '^issue edit 21 ' "$gh_log")" -eq 1 ]
  [ "$(grep -c '^issue close 21 ' "$gh_log")" -eq 1 ]
}
