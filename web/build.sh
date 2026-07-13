#!/usr/bin/env bash
# Build an example for the web:  ./web/build.sh [example]   (default: imgui)
#
# dear imgui's C++ is cross-compiled into the same wasm module with the
# wasi-sdk toolchain (https://github.com/WebAssembly/wasi-sdk/releases).
# Point WASI_SDK_PATH at the extracted SDK; defaults to ~/.wasi-sdk.
# The IMGUI_SYS_WASM_CC / IMGUI_SYS_SINGLE_MODULE switches live in
# .cargo/config.toml (they are path-free and also serve rust-analyzer).
set -euo pipefail
cd "$(dirname "$0")/.."

EXAMPLE="${1:-imgui}"

SDK="${WASI_SDK_PATH:-$HOME/.wasi-sdk}"
# Git Bash / MSYS: hand rustc and clang C:/-style paths, not /c/-style.
if command -v cygpath >/dev/null 2>&1; then SDK="$(cygpath -m "$SDK")"; fi
if [ ! -d "$SDK" ]; then
  echo "error: wasi-sdk not found at '$SDK' — set WASI_SDK_PATH" >&2
  exit 1
fi

EXE=""
case "$(uname -s)" in MINGW*|MSYS*|CYGWIN*) EXE=".exe" ;; esac

SYSROOT="$SDK/share/wasi-sysroot"

# wasi-sdk keeps libc headers under the wasip1 triple; the -isystem makes
# them visible when compiling with --target=wasm32-unknown-unknown.
# NDEBUG drops IM_ASSERT (musl's __assert_fail would drag in fd_write and
# other WASI imports the browser can't provide); the IMGUI_DISABLE_*
# defines and the IMGUI_DEBUG_PRINTF stub cut the remaining stdio surface.
FLAGS="-isystem $SYSROOT/include/wasm32-wasip1 -DNDEBUG"

# (cc-rs accepts the underscore form of the target in these var names.)
export CC_wasm32_unknown_unknown="$SDK/bin/clang$EXE"
export CXX_wasm32_unknown_unknown="$SDK/bin/clang++$EXE"
export AR_wasm32_unknown_unknown="$SDK/bin/ar$EXE"
# dear imgui uses no STL and builds with -fno-exceptions; skip the C++
# standard library entirely instead of linking libstdc++/libc++.
export CXXSTDLIB_wasm32_unknown_unknown=""
export CFLAGS_wasm32_unknown_unknown="$FLAGS"
export CXXFLAGS_wasm32_unknown_unknown="$FLAGS -DIMGUI_DISABLE_FILE_FUNCTIONS -DIMGUI_DISABLE_DEFAULT_SHELL_FUNCTIONS -DIMGUI_DEBUG_PRINTF(...)=((void)0)"
# The C++'s libc calls (vsnprintf, sscanf, cosf, ...) are satisfied by
# linking wasi-libc's static archive.
export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="-C link-arg=$SYSROOT/lib/wasm32-wasip1/libc.a"

cargo build --release --example "$EXAMPLE" --target wasm32-unknown-unknown
wasm-bindgen "target/wasm32-unknown-unknown/release/examples/$EXAMPLE.wasm" \
  --out-dir web/pkg --target web --no-typescript

echo "built web/pkg/$EXAMPLE.js — serve the web/ directory"
