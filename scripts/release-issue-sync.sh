#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

from=${1:?usage: release-issue-sync.sh <from-revision> <to-revision> <version> <release-url>}
to=${2:?usage: release-issue-sync.sh <from-revision> <to-revision> <version> <release-url>}
version=${3:?usage: release-issue-sync.sh <from-revision> <to-revision> <version> <release-url>}
release_url=${4:?usage: release-issue-sync.sh <from-revision> <to-revision> <version> <release-url>}
repository=${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must name the published repository}

trailers=$(mktemp)
trap 'rm -f "$trailers"' EXIT

git log --format=%B "$from..$to" \
  | awk '
      /^Release-Updates-Issue:[[:space:]]*[0-9]+[[:space:]]*$/ {
        sub(/^Release-Updates-Issue:[[:space:]]*/, "")
        print "update:" $0
      }
      /^Release-Closes-Issue:[[:space:]]*[0-9]+[[:space:]]*$/ {
        sub(/^Release-Closes-Issue:[[:space:]]*/, "")
        print "close:" $0
      }
    ' \
  | sort -u >"$trailers"

issue_numbers=$(cut -d: -f2 "$trailers" | sort -un)
for issue in $issue_numbers; do
  marker="<!-- emc-release:${version}:issue-${issue} -->"
  comments=$(gh api --paginate "repos/${repository}/issues/${issue}/comments" --jq '.[].body')
  if ! printf '%s\n' "$comments" | grep -Fqx "$marker"; then
    comment_body=$(printf 'Released in %s: %s\n\n%s' "$version" "$release_url" "$marker")
    gh issue comment "$issue" --repo "$repository" \
      --body "$comment_body"
  fi

  if grep -Fqx "close:${issue}" "$trailers"; then
    labels=$(gh issue view "$issue" --repo "$repository" --json labels --jq '.labels[].name')
    state=$(gh issue view "$issue" --repo "$repository" --json state --jq '.state')
    if printf '%s\n' "$labels" | grep -Fqx 'status:accepted'; then
      if printf '%s\n' "$labels" | grep -Fqx 'status:released'; then
        gh issue edit "$issue" --repo "$repository" --remove-label status:accepted
      else
        gh issue edit "$issue" --repo "$repository" \
          --add-label status:released --remove-label status:accepted
      fi
    elif ! printf '%s\n' "$labels" | grep -Fqx 'status:released'; then
      gh issue edit "$issue" --repo "$repository" \
        --add-label status:released --remove-label status:accepted
    fi
    if [ "$state" = OPEN ]; then
      gh issue close "$issue" --repo "$repository" --reason completed
    fi
  fi
done
