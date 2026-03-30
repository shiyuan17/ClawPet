#!/usr/bin/env bash

set -euo pipefail

TARGET="${1:-x86_64-pc-windows-msvc}"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v cargo-xwin >/dev/null 2>&1; then
  echo "error: cargo-xwin is required. Install it with: cargo install cargo-xwin"
  exit 1
fi

llvm_bin=""
if command -v llvm-lib >/dev/null 2>&1; then
  llvm_bin="$(dirname "$(command -v llvm-lib)")"
elif command -v brew >/dev/null 2>&1; then
  llvm_prefix="$(brew --prefix llvm 2>/dev/null || true)"
  if [ -n "${llvm_prefix}" ] && [ -x "${llvm_prefix}/bin/llvm-lib" ]; then
    llvm_bin="${llvm_prefix}/bin"
  fi
fi

if [ -z "${llvm_bin}" ]; then
  cat <<'EOF'
error: missing llvm-lib

Cross-compiling Tauri for Windows (msvc target) requires LLVM tools.
Install LLVM and expose it in PATH, then run build again:

  brew install llvm
  export PATH="$(brew --prefix llvm)/bin:$PATH"
EOF
  exit 1
fi

nsis_path=""
if [ -x "/opt/homebrew/bin/makensis" ]; then
  nsis_path="/opt/homebrew/bin/makensis"
elif [ -x "/usr/local/bin/makensis" ]; then
  nsis_path="/usr/local/bin/makensis"
fi

export PATH="${llvm_bin}:${HOME}/.cargo/bin:${PATH}"

# Keep Windows cross-compilation artifacts isolated from regular host builds.
# This avoids stale/mixed metadata issues (e.g. "crate `itoa` ... rlib format").
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-${PROJECT_ROOT}/src-tauri/target/tauri-win}"

# Rust 1.94 + cargo-xwin can fail in release mode for proc-macro deps with:
# "crate `quote` required to be available in rlib format, but was not found".
# Disable release stripping for this build path to avoid the broken code path.
export CARGO_PROFILE_RELEASE_STRIP="${CARGO_PROFILE_RELEASE_STRIP:-none}"

if [ -n "${nsis_path}" ]; then
  CI=false NSIS_PATH="${nsis_path}" tauri build --runner cargo-xwin --target "${TARGET}" --config src-tauri/tauri.windows.conf.json
else
  CI=false tauri build --runner cargo-xwin --target "${TARGET}" --config src-tauri/tauri.windows.conf.json
fi
