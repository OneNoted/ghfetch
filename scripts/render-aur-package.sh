#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF' >&2
usage:
  render-aur-package.sh ghfetch-rs-git <output-dir> [--pkgver VERSION] [--pkgrel N]
  render-aur-package.sh ghfetch-rs-bin <output-dir> --pkgver VERSION --sha256 SHA256 [--pkgrel N]
EOF
  exit 1
}

if [[ $# -lt 2 ]]; then
  usage
fi

package_name="$1"
output_dir="$2"
shift 2

pkgver=""
pkgrel="1"
archive_sha256=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pkgver)
      pkgver="$2"
      shift 2
      ;;
    --pkgrel)
      pkgrel="$2"
      shift 2
      ;;
    --sha256)
      archive_sha256="$2"
      shift 2
      ;;
    *)
      usage
      ;;
  esac
done

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
template_path="${repo_root}/packaging/aur/${package_name}/PKGBUILD.in"

if [[ ! -f "${template_path}" ]]; then
  echo "unknown package template: ${package_name}" >&2
  exit 1
fi

if [[ -z "${pkgver}" ]]; then
  if [[ "${package_name}" == "ghfetch-rs-git" ]]; then
    pkgver="$("${repo_root}/scripts/compute-git-pkgver.sh" "${repo_root}")"
  else
    echo "--pkgver is required for ${package_name}" >&2
    exit 1
  fi
fi

if [[ "${package_name}" == "ghfetch-rs-bin" && -z "${archive_sha256}" ]]; then
  echo "--sha256 is required for ghfetch-rs-bin" >&2
  exit 1
fi

maintainer_name="${AUR_MAINTAINER_NAME:-Jonatan Jonasson}"
maintainer_email="${AUR_MAINTAINER_EMAIL:-notes@madeingotland.com}"
repo_slug="${AUR_REPO_SLUG:-OneNoted/ghfetch}"
repo_url="${AUR_REPO_URL:-https://github.com/OneNoted/ghfetch.git}"
archive_root="ghfetch-${pkgver}-x86_64-unknown-linux-gnu"
archive_name="${archive_root}.tar.gz"

mkdir -p "${output_dir}"

export AUR_MAINTAINER_NAME="${maintainer_name}"
export AUR_MAINTAINER_EMAIL="${maintainer_email}"
export AUR_PKGNAME="${package_name}"
export AUR_PKGVER="${pkgver}"
export AUR_PKGREL="${pkgrel}"
export AUR_REPO_SLUG="${repo_slug}"
export AUR_REPO_URL="${repo_url}"
export AUR_ARCHIVE_ROOT="${archive_root}"
export AUR_ARCHIVE_NAME="${archive_name}"
export AUR_ARCHIVE_SHA256="${archive_sha256}"

envsubst \
  '${AUR_MAINTAINER_NAME} ${AUR_MAINTAINER_EMAIL} ${AUR_PKGNAME} ${AUR_PKGVER} ${AUR_PKGREL} ${AUR_REPO_SLUG} ${AUR_REPO_URL} ${AUR_ARCHIVE_ROOT} ${AUR_ARCHIVE_NAME} ${AUR_ARCHIVE_SHA256}' \
  < "${template_path}" \
  > "${output_dir}/PKGBUILD"

(
  cd "${output_dir}"
  makepkg --printsrcinfo > .SRCINFO
)
