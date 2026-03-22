#!/bin/sh
# skillx installer — downloads the latest release binary for your platform.
# Usage: curl -fsSL https://skillx.run/install.sh | sh

set -e

REPO="skillx-run/skillx"
INSTALL_DIR="${HOME}/.local/bin"

# Verify SHA256 checksum of the downloaded archive.
# Gracefully degrades if checksum file is unavailable or no hash tool is found.
verify_checksum() {
    archive_path="$1"
    archive_name="$2"

    checksum_url="https://github.com/${REPO}/releases/latest/download/sha256sums.txt"

    # Download checksums file
    checksums_file="${_SKILLX_TMP}/sha256sums.txt"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$checksum_url" -o "$checksums_file" 2>/dev/null || true
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$checksums_file" "$checksum_url" 2>/dev/null || true
    fi

    if [ ! -f "$checksums_file" ] || [ ! -s "$checksums_file" ]; then
        echo "Warning: could not download checksums file, skipping verification" >&2
        return 0
    fi

    # Extract expected hash for our archive
    expected_hash=$(grep "${archive_name}" "$checksums_file" | awk '{print $1}')
    if [ -z "$expected_hash" ]; then
        echo "Warning: no checksum found for ${archive_name}, skipping verification" >&2
        return 0
    fi

    # Compute actual hash
    if command -v shasum >/dev/null 2>&1; then
        actual_hash=$(shasum -a 256 "$archive_path" | awk '{print $1}')
    elif command -v sha256sum >/dev/null 2>&1; then
        actual_hash=$(sha256sum "$archive_path" | awk '{print $1}')
    else
        echo "Warning: no SHA256 tool available, skipping verification" >&2
        return 0
    fi

    if [ "$expected_hash" != "$actual_hash" ]; then
        echo "Error: SHA256 checksum mismatch!" >&2
        echo "  Expected: ${expected_hash}" >&2
        echo "  Actual:   ${actual_hash}" >&2
        echo "The downloaded file may be corrupted or tampered with." >&2
        exit 1
    fi

    echo "Checksum verified."
}

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
    _SKILLX_TMP="$(mktemp -d)"
    trap 'rm -rf "$_SKILLX_TMP"' EXIT

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$URL" -o "${_SKILLX_TMP}/${ARCHIVE}"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "${_SKILLX_TMP}/${ARCHIVE}" "$URL"
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi

    # Verify checksum before extraction
    verify_checksum "${_SKILLX_TMP}/${ARCHIVE}" "${ARCHIVE}"

    echo "Extracting..."
    tar -xzf "${_SKILLX_TMP}/${ARCHIVE}" -C "$_SKILLX_TMP"

    # Find the binary (handles both flat and directory-wrapped archives)
    BINARY="$(find "$_SKILLX_TMP" -name skillx -type f ! -name '*.gz' | head -1)"
    if [ -z "$BINARY" ]; then
        echo "Error: could not find skillx binary in archive" >&2
        exit 1
    fi

    # Install binary
    mkdir -p "$INSTALL_DIR"
    mv "$BINARY" "${INSTALL_DIR}/skillx"
    chmod +x "${INSTALL_DIR}/skillx"

    echo "Installed skillx to ${INSTALL_DIR}/skillx"

    # Verify
    if "${INSTALL_DIR}/skillx" --version >/dev/null 2>&1; then
        echo "[ok] $("${INSTALL_DIR}/skillx" --version)"
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
