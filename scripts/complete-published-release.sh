#!/bin/sh
# Copyright 2026 John Wilger

set -eu

version="${1:?release version is required}"
tag="v${version}"

if ! git ls-remote --exit-code --tags origin "refs/tags/${tag}" >/dev/null 2>&1; then
  git tag -s "$tag" -m "$tag" HEAD
  git push origin "refs/tags/${tag}"
fi

if ! gh release view "$tag" >/dev/null 2>&1; then
  gh release create "$tag" --target HEAD --title "$tag" --generate-notes
fi
