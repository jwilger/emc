#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

if [ -z "${GIT_TOKEN:-}" ]; then
  echo "GIT_TOKEN must be set before configuring release-plz git auth" >&2
  exit 1
fi

askpass_path="${RUNNER_TEMP:-/tmp}/release-plz-git-askpass"
mkdir -p "$(dirname "$askpass_path")"

cat > "$askpass_path" <<'SH'
#!/usr/bin/env sh

case "$1" in
  *Username*)
    printf '%s\n' "${RELEASE_PLZ_GIT_USERNAME:-jwilger}"
    ;;
  *Password*)
    printf '%s\n' "$GIT_TOKEN"
    ;;
  *)
    printf '\n'
    ;;
esac
SH

chmod 700 "$askpass_path"

export GIT_ASKPASS="$askpass_path"
export GIT_TERMINAL_PROMPT=0
