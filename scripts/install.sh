#!/bin/bash
# Sage installer
# Usage: curl -fsSL https://raw.githubusercontent.com/sagelang/sage/main/scripts/install.sh | bash

set -euo pipefail

REPO="sagelang/sage"
INSTALL_DIR="${SAGE_INSTALL_DIR:-/usr/local/sage}"
TMPDIR_CLEANUP=""

cleanup() {
    if [ -n "$TMPDIR_CLEANUP" ] && [ -d "$TMPDIR_CLEANUP" ]; then
        rm -rf "$TMPDIR_CLEANUP"
    fi
}
trap cleanup EXIT INT TERM

# Detect platform
detect_platform() {
    local os arch target

    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            case "$arch" in
                arm64) target="aarch64-apple-darwin" ;;
                x86_64) target="x86_64-apple-darwin" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        Linux)
            case "$arch" in
                x86_64) target="x86_64-unknown-linux-gnu" ;;
                aarch64) target="aarch64-unknown-linux-gnu" ;;
                *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
            esac
            ;;
        *)
            echo "Unsupported OS: $os" >&2
            exit 1
            ;;
    esac

    echo "$target"
}

# Get latest version
get_latest_version() {
    curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | cut -d'"' -f4
}

main() {
    echo "🦉 Installing Sage..."
    echo

    local target version url

    target="$(detect_platform)"
    echo "  Platform: $target"

    version="${SAGE_VERSION:-$(get_latest_version)}"
    if [ -z "$version" ]; then
        echo "Error: Could not determine latest version" >&2
        echo "No releases found. Build from source:" >&2
        echo "  git clone https://github.com/$REPO && cd sage && cargo build --release" >&2
        exit 1
    fi
    echo "  Version:  $version"

    url="https://github.com/$REPO/releases/download/$version/sage-$version-$target.tar.gz"
    echo "  URL:      $url"
    echo

    # Download and extract
    echo "📦 Downloading..."
    local tmpdir
    tmpdir="$(mktemp -d)"
    TMPDIR_CLEANUP="$tmpdir"

    curl -fsSL "$url" | tar xz -C "$tmpdir"

    # Install
    echo "📂 Installing to $INSTALL_DIR..."
    sudo rm -rf "$INSTALL_DIR"
    sudo mv "$tmpdir/sage-$version-$target" "$INSTALL_DIR"
    sudo ln -sf "$INSTALL_DIR/bin/sage" /usr/local/bin/sage

    # Verify
    if command -v sage &>/dev/null; then
        echo
        echo "✅ Sage installed successfully!"
        echo
        sage --version
        echo
        echo "Add this to your shell profile for fast builds:"
        echo "  export SAGE_TOOLCHAIN=$INSTALL_DIR/toolchain"
        echo
        echo "Get started:"
        echo "  sage run examples/hello.sg"
    else
        echo
        echo "⚠️  Installation complete, but 'sage' not found in PATH"
        echo "  Add /usr/local/bin to your PATH"
    fi
}

main "$@"
