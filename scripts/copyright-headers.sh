#!/usr/bin/env sh
# Copyright 2026 John Wilger

set -eu

copyright_text='Copyright 2026 John Wilger'

usage() {
  printf '%s\n' 'usage: scripts/copyright-headers.sh (--check|--fix) [file ...]' >&2
}

mode="${1:-}"
case "$mode" in
  --check | --fix)
    shift
    ;;
  *)
    usage
    exit 2
    ;;
esac

is_supported_file() {
  case "$1" in
    Cargo.lock | flake.lock | LICENSE)
      return 1
      ;;
    *.rs | *.md | *.feature | *.nix | *.toml | *.yml | *.yaml | *.sh | justfile | AGENTS.md | README.md)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

line_comment_prefix() {
  case "$1" in
    *.rs)
      printf '%s\n' '//'
      ;;
    *)
      printf '%s\n' '#'
      ;;
  esac
}

header_for_file() {
  case "$1" in
    *.md)
      printf '<!-- %s -->\n' "$copyright_text"
      ;;
    *)
      printf '%s %s\n' "$(line_comment_prefix "$1")" "$copyright_text"
      ;;
  esac
}

has_header() {
  sed -n '1,5p' "$1" | grep -Fq "$copyright_text"
}

add_header() {
  file="$1"
  header="$(header_for_file "$file")"
  tmp="$(mktemp)"
  first_line="$(sed -n '1p' "$file")"

  if printf '%s\n' "$first_line" | grep -q '^#!'; then
    {
      printf '%s\n' "$first_line"
      printf '%s\n\n' "$header"
      sed '1d' "$file"
    } > "$tmp"
  else
    {
      printf '%s\n\n' "$header"
      cat "$file"
    } > "$tmp"
  fi

  mv "$tmp" "$file"
}

if [ "$#" -gt 0 ]; then
  files="$*"
else
  files="$(git ls-files)"
fi

missing=0
for file in $files; do
  if [ ! -f "$file" ] || ! is_supported_file "$file"; then
    continue
  fi

  if has_header "$file"; then
    continue
  fi

  if [ "$mode" = "--fix" ]; then
    add_header "$file"
  else
    printf '%s: missing copyright header\n' "$file" >&2
    missing=1
  fi
done

exit "$missing"
