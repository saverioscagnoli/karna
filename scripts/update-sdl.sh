#!/usr/bin/env bash

#
# Bump the vendored SDL submodule to the latest STABLE upstream release.
#
# SDL uses parity versioning: a version is a prerelease if either the minor
# or the micro component is odd. GitHub's `prerelease` flag is not reliable
# for this on its own, so we filter on parity as well.
#
# Usage:
#   scripts/update-sdl.sh            # bump to latest stable
#   scripts/update-sdl.sh --check    # report only, change nothing
#   scripts/update-sdl.sh 3.4.14     # pin an explicit version

set -euo pipefail

REPO="libsdl-org/SDL"
VENDOR="vendor/SDL"
BUILD_RS="build.rs"

die() { printf 'error: %s\n' "$*" >&2; exit 1; }

for tool in curl jq git awk sort; do
    command -v "$tool" >/dev/null 2>&1 || die "missing required tool: $tool"
done

cd "$(git rev-parse --show-toplevel)"
[ -d "$VENDOR" ] || die "$VENDOR not found; run: git submodule update --init"

latest_stable() {
    local auth=()
    # Avoid the 60 req/hr unauthenticated limit in CI.
    [ -n "${GITHUB_TOKEN:-}" ] && auth=(-H "Authorization: Bearer $GITHUB_TOKEN")

    curl -fsSL "${auth[@]}" \
        "https://api.github.com/repos/$REPO/releases?per_page=100" \
    | jq -r '.[] | select(.draft == false and .prerelease == false) | .tag_name' \
    | sed -n 's/^release-//p' \
    | awk -F. 'NF == 3 && $1 == 3 && $2 % 2 == 0 && $3 % 2 == 0' \
    | sort -V \
    | tail -n 1
}

current_version() {
    awk '
        /#define[[:space:]]+SDL_MAJOR_VERSION[[:space:]]/ { maj = $3 }
        /#define[[:space:]]+SDL_MINOR_VERSION[[:space:]]/ { min = $3 }
        /#define[[:space:]]+SDL_MICRO_VERSION[[:space:]]/ { mic = $3 }
        END { if (maj != "") print maj "." min "." mic }
    ' "$VENDOR/include/SDL3/SDL_version.h" 2>/dev/null || true
}

target="${1:-}"
check_only=false

if [ "$target" = "--check" ]; then
    check_only=true
    target=""
fi

if [ -z "$target" ]; then
    target="$(latest_stable)"
    [ -n "$target" ] || die "could not determine latest stable release"
fi

current="$(current_version)"
printf 'current: %s\nlatest:  %s\n' "${current:-<none>}" "$target"

if [ "$current" = "$target" ]; then
    echo "already up to date"
    exit 0
fi

if [ "$check_only" = true ]; then
    echo "update available (run without --check to apply)"
    exit 1
fi

echo "==> fetching tags"
git -C "$VENDOR" fetch --tags --quiet origin

git -C "$VENDOR" rev-parse -q --verify "refs/tags/release-$target" >/dev/null \
    || die "tag release-$target does not exist upstream"

echo "==> checking out release-$target"
git -C "$VENDOR" checkout --quiet "release-$target"

echo "==> updating SDL_MIN_VERSION in $BUILD_RS"
if [ -f "$BUILD_RS" ]; then
    tmp="$(mktemp)"
    sed -E "s|^(const SDL_MIN_VERSION: &str = )\"[^\"]*\";|\\1\"$target\";|" \
        "$BUILD_RS" > "$tmp"
    mv "$tmp" "$BUILD_RS"
fi

cat <<EOF

Updated to SDL $target.

Next:
  cargo build --features bundled
  cargo test
  git add $VENDOR $BUILD_RS
  git commit -m "deps: bump SDL to $target"
EOF
