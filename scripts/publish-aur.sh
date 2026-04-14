#!/usr/bin/env bash

set -euo pipefail

if [[ $# -lt 2 || $# -gt 3 ]]; then
  echo "usage: $0 <aur-package-name> <rendered-package-dir> [commit-message]" >&2
  exit 1
fi

package_name="$1"
rendered_dir="$2"
commit_message="${3:-chore: update ${package_name}}"
author_name="${AUR_GIT_AUTHOR_NAME:-Jonatan Jonasson}"
author_email="${AUR_GIT_AUTHOR_EMAIL:-notes@madeingotland.com}"

if [[ ! -f "${rendered_dir}/PKGBUILD" || ! -f "${rendered_dir}/.SRCINFO" ]]; then
  echo "rendered package directory must contain PKGBUILD and .SRCINFO" >&2
  exit 1
fi

checkout_dir="$(mktemp -d)"
trap 'rm -rf "${checkout_dir}"' EXIT

git clone "ssh://aur@aur.archlinux.org/${package_name}.git" "${checkout_dir}"

find "${checkout_dir}" -mindepth 1 -maxdepth 1 ! -name '.git' -exec rm -rf {} +

shopt -s dotglob nullglob
for path in "${rendered_dir}"/* "${rendered_dir}"/.*; do
  name="$(basename "${path}")"
  case "${name}" in
    .|..|*.in)
      continue
      ;;
  esac
  cp -a "${path}" "${checkout_dir}/${name}"
done
shopt -u dotglob nullglob

cd "${checkout_dir}"

git config user.name "${author_name}"
git config user.email "${author_email}"
git add --all

if git diff --cached --quiet; then
  echo "No AUR changes to publish for ${package_name}"
  exit 0
fi

git commit -m "${commit_message}"
git push origin HEAD:master
