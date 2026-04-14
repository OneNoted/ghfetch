#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 <version> <output-dir>" >&2
  exit 1
fi

version="$1"
output_dir="$2"
target_triple="${TARGET_TRIPLE:-x86_64-unknown-linux-gnu}"
source_date_epoch="${SOURCE_DATE_EPOCH:-0}"

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
archive_root="ghfetch-${version}-${target_triple}"
archive_path="${output_dir}/${archive_root}.tar.gz"
tar_path="${output_dir}/${archive_root}.tar"

cd "${repo_root}"

mkdir -p "${output_dir}"

cargo build --locked --release --target "${target_triple}"

staging_dir="$(mktemp -d)"
trap 'rm -rf "${staging_dir}"' EXIT

install -Dm755 "target/${target_triple}/release/ghfetch" "${staging_dir}/${archive_root}/ghfetch"
install -Dm644 LICENSE "${staging_dir}/${archive_root}/LICENSE"

tar \
  --sort=name \
  --owner=0 \
  --group=0 \
  --numeric-owner \
  --mtime="@${source_date_epoch}" \
  -cf "${tar_path}" \
  -C "${staging_dir}" \
  "${archive_root}"

gzip -n -f "${tar_path}"
sha256sum "${archive_path}" > "${archive_path}.sha256"

printf '%s\n' "${archive_path}"
