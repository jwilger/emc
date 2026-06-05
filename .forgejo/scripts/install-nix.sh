#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

NIX_BIN_DIR="/nix/var/nix/profiles/default/bin"

if [ -x "$NIX_BIN_DIR/nix" ]; then
  echo "Nix already present at $NIX_BIN_DIR (volume cache hit)"
else
  echo "Installing Nix (cold start or fresh volume)"
  curl --proto '=https' --tlsv1.2 -sSf -L \
    https://install.determinate.systems/nix \
    | sh -s -- install linux \
      --init none \
      --no-confirm \
      --extra-conf "experimental-features = nix-command flakes" \
      --extra-conf "accept-flake-config = true"
fi

if ! getent group nixbld > /dev/null; then
  groupadd -r nixbld
fi

for i in $(seq 1 32); do
  if ! id -u "nixbld$i" > /dev/null 2>&1; then
    useradd -r -g nixbld -G nixbld -d /var/empty \
      -s /usr/sbin/nologin -c "Nix build user $i" "nixbld$i"
  fi
done

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "$NIX_BIN_DIR" >> "$GITHUB_PATH"
fi
