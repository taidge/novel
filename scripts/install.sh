#!/usr/bin/env bash
#
# Novel installer — downloads a prebuilt binary from GitHub releases.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/taidge/novel/main/scripts/install.sh | bash
#
# Environment variables:
#   NOVEL_VERSION   Pin a specific version (e.g. "v0.1.0"). Default: latest.
#   NOVEL_DIR       Install directory. Default: $HOME/.local/bin.
#   NOVEL_REPO      Override the GitHub repo. Default: taidge/novel.

set -euo pipefail

REPO="${NOVEL_REPO:-taidge/novel}"
INSTALL_DIR="${NOVEL_DIR:-$HOME/.local/bin}"
BIN_NAME="novel"

log() { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33mwarning:\033[0m %s\n' "$*" >&2; }
err() { printf '\033[1;31merror:\033[0m %s\n' "$*" >&2; exit 1; }

need() {
  command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

need curl
need tar
need uname
need mkdir
need install

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}" in
    Linux)   os="unknown-linux-gnu" ;;
    Darwin)  os="apple-darwin" ;;
    *)       err "unsupported OS: ${os}" ;;
  esac

  case "${arch}" in
    x86_64 | amd64)   arch="x86_64" ;;
    arm64 | aarch64)  arch="aarch64" ;;
    *)                err "unsupported arch: ${arch}" ;;
  esac

  printf '%s-%s' "${arch}" "${os}"
}

resolve_version() {
  if [ -n "${NOVEL_VERSION:-}" ]; then
    printf '%s' "${NOVEL_VERSION}"
    return
  fi
  # GitHub's "latest" redirect returns the tag in the final URL.
  local url
  url=$(curl -fsSLI -o /dev/null -w '%{url_effective}' \
    "https://github.com/${REPO}/releases/latest") || err "failed to resolve latest version"
  printf '%s' "${url##*/}"
}

main() {
  local target version tag asset url tmp stage

  target="$(detect_target)"
  tag="$(resolve_version)"
  version="${tag#v}"
  asset="novel-v${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

  log "target: ${target}"
  log "version: ${tag}"
  log "downloading: ${url}"

  tmp=$(mktemp -d)
  trap 'rm -rf "${tmp}"' EXIT

  if ! curl -fsSL "${url}" -o "${tmp}/${asset}"; then
    err "failed to download ${asset}. Does this release include a build for ${target}?"
  fi

  log "extracting"
  tar -xzf "${tmp}/${asset}" -C "${tmp}"
  stage="${tmp}/novel-v${version}-${target}"
  [ -f "${stage}/${BIN_NAME}" ] || err "archive did not contain ${BIN_NAME}"

  mkdir -p "${INSTALL_DIR}"
  install -m 0755 "${stage}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

  log "installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"

  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
      "${INSTALL_DIR}/${BIN_NAME}" --version || true
      ;;
    *)
      warn "${INSTALL_DIR} is not on your PATH."
      warn "Add it by appending this line to your shell profile:"
      printf '\n    export PATH="%s:$PATH"\n\n' "${INSTALL_DIR}"
      ;;
  esac
}

main "$@"
