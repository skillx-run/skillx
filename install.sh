#!/bin/sh
# skillx installer — downloads the latest release binary for your platform.
# Usage: curl -fsSL https://skillx.run/install.sh | sh

set -e

REPO="anthropics/skillx"
INSTALL_DIR="${HOME}/.local/bin"

main() {
    # Detect OS
    OS="$(uname -s)"
    case "$OS" in
        Linux)  os="unknown-linux-gnu" ;;
        Darwin) os="apple-darwin" ;;
        *)
            echo "Error: unsupported OS: $OS" >&2
            exit 1
            ;;
    esac

    # Detect architecture
    ARCH="$(uname -m)"
    case "$ARCH" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *)
            echo "Error: unsupported architecture: $ARCH" >&2
            exit 1
            ;;
    esac

    TARGET="${arch}-${os}"
    ARCHIVE="skillx-${TARGET}.tar.gz"
    URL="https://github.com/${REPO}/releases/latest/download/${ARCHIVE}"

    echo "Downloading skillx for ${TARGET}..."
    TMPDIR="$(mktemp -d)"
    trap 'rm -rf "$TMPDIR"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$URL" -o "${TMPDIR}/${ARCHIVE}"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "${TMPDIR}/${ARCHIVE}" "$URL"
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi

    echo "Extracting..."
    tar -xzf "${TMPDIR}/${ARCHIVE}" -C "$TMPDIR"

    # Install binary
    mkdir -p "$INSTALL_DIR"
    mv "${TMPDIR}/skillx" "${INSTALL_DIR}/skillx"
    chmod +x "${INSTALL_DIR}/skillx"

    echo "Installed skillx to ${INSTALL_DIR}/skillx"

    # Verify
    if "${INSTALL_DIR}/skillx" --version >/dev/null 2>&1; then
        echo "✓ $("${INSTALL_DIR}/skillx" --version)"
    fi

    # Check PATH
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*) ;;
        *)
            echo ""
            echo "Add ${INSTALL_DIR} to your PATH:"
            echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
            echo ""
            echo "Or add it to your shell profile (~/.bashrc, ~/.zshrc, etc.)."
            ;;
    esac
}

main
