#!/bin/bash
set -euo pipefail

REPO="jiweiyuan/talkd"
INSTALL_DIR="/usr/local/bin"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Darwin) OS_LABEL="apple-darwin" ;;
  Linux)  OS_LABEL="unknown-linux-gnu" ;;
  *)      echo "Unsupported OS: $OS"; exit 1 ;;
esac

case "$ARCH" in
  arm64|aarch64) ARCH_LABEL="aarch64" ;;
  x86_64)        ARCH_LABEL="x86_64" ;;
  *)             echo "Unsupported architecture: $ARCH"; exit 1 ;;
esac

TARGET="${ARCH_LABEL}-${OS_LABEL}"
ASSET="talkd-${TARGET}.tar.gz"

# Get latest release tag
TAG=$(curl -sI "https://github.com/${REPO}/releases/latest" | grep -i "^location:" | sed 's/.*tag\///' | tr -d '\r\n')
if [ -z "$TAG" ]; then
  TAG="v0.3.2"
fi

URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"

echo "Installing talkd ${TAG} (${TARGET})..."

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -sSL "$URL" -o "$TMPDIR/$ASSET"
tar xzf "$TMPDIR/$ASSET" -C "$TMPDIR"

if [ -w "$INSTALL_DIR" ]; then
  mv "$TMPDIR/talkd" "$INSTALL_DIR/talkd"
else
  echo "Need sudo to install to $INSTALL_DIR"
  sudo mv "$TMPDIR/talkd" "$INSTALL_DIR/talkd"
fi

echo "talkd installed to $INSTALL_DIR/talkd"
talkd --version
