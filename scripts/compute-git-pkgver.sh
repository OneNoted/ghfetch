#!/usr/bin/env bash

set -euo pipefail

repo_root="${1:-.}"

cd "${repo_root}"

description="$(git describe --long --tags --abbrev=7 --match 'v[0-9]*' 2>/dev/null || true)"

if [[ -n "${description}" ]]; then
  description="${description#v}"
  printf '%s\n' "${description}" | sed 's/\([^-]*-g\)/r\1/; s/-/./g'
  exit 0
fi

cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"

printf '%s.r%s.g%s\n' \
  "${cargo_version}" \
  "$(git rev-list --count HEAD)" \
  "$(git rev-parse --short=7 HEAD)"
