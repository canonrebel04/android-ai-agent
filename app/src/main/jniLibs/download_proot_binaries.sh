#!/bin/sh
# download_proot_binaries.sh
# Script to download PRoot binaries for Android
# Usage: ./download_proot_binaries.sh [VERSION]
# Example: ./download_proot_binaries.sh v5.3.0

set -e

# Default version (check for latest at https://github.com/termux/proot/releases)
DEFAULT_VERSION="v5.3.0"
VERSION="${1:-$DEFAULT_VERSION}"

# Base URL for Termux PRoot releases
BASE_URL="https://github.com/termux/proot/releases/download/${VERSION}"

# Target directories
JNI_LIBS_DIR="$(dirname "$0")"
ARM64_DIR="${JNI_LIBS_DIR}/arm64-v8a"
ARMv7_DIR="${JNI_LIBS_DIR}/armeabi-v7a"
X86_64_DIR="${JNI_LIBS_DIR}/x86_64"

# Binary names (as they appear in releases)
ARM64_BINARY="proot-aarch64"
ARMv7_BINARY="proot-arm"
X86_64_BINARY="proot-x86_64"

echo "Downloading PRoot binaries version ${VERSION}..."
echo "Target directory: ${JNI_LIBS_DIR}"

# Create directories if they don't exist
mkdir -p "${ARM64_DIR}" "${ARMv7_DIR}" "${X86_64_DIR}"

# Download function
download_binary() {
    local url="$1"
    local target="$2"
    local binary_name="$3"
    
    echo "Downloading ${binary_name}..."
    if command -v wget >/dev/null 2>&1; then
        wget -q "${url}" -O "${target}"
    elif command -v curl >/dev/null 2>&1; then
        curl -L -s "${url}" -o "${target}"
    else
        echo "ERROR: Neither wget nor curl found. Please install one of them."
        exit 1
    fi
    
    if [ -f "${target}" ]; then
        chmod +x "${target}"
        echo "  -> Saved to ${target}"
    else
        echo "ERROR: Failed to download ${binary_name}"
        exit 1
    fi
}

# Download binaries
download_binary "${BASE_URL}/${ARM64_BINARY}" "${ARM64_DIR}/libproot.so" "ARM64 binary"
download_binary "${BASE_URL}/${ARMv7_BINARY}" "${ARMv7_DIR}/libproot.so" "ARMv7 binary"
download_binary "${BASE_URL}/${X86_64_BINARY}" "${X86_64_DIR}/libproot.so" "x86_64 binary"

echo ""
echo "All PRoot binaries downloaded successfully!"
echo ""
echo "Verifying binaries..."

# Verify file types
verify_binary() {
    local file="$1"
    local expected="$2"
    
    if command -v file >/dev/null 2>&1; then
        local file_type=$(file "${file}")
        if echo "${file_type}" | grep -q "${expected}"; then
            echo "  ✓ ${file}: ${file_type}"
        else
            echo "  ✗ ${file}: Unexpected file type - ${file_type}"
        fi
    else
        echo "  ? ${file}: (file command not available, skipping verification)"
    fi
}

echo "ARM64 binary:"
verify_binary "${ARM64_DIR}/libproot.so" "ARM aarch64"

echo "ARMv7 binary:"
verify_binary "${ARMv7_DIR}/libproot.so" "ARM"

echo "x86_64 binary:"
verify_binary "${X86_64_DIR}/libproot.so" "x86-64"

echo ""
echo "Done!"
