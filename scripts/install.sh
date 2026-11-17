#!/bin/sh
# Fenneth installer for Linux, macOS and Termux (Android).
#
#   curl -fsSL https://github.com/__REPO__/releases/latest/download/install.sh | sh
#
# Environment variables:
#   FENNETH_VERSION        Release tag to install, e.g. 0.003 (default: latest)
#   FENNETH_INSTALL        Install root; the binary goes to $FENNETH_INSTALL/bin
#                          (default: $HOME/.fenneth, or $PREFIX on Termux)
#   FENNETH_NO_MODIFY_PATH Set to 1 to leave your shell profiles untouched
#   FENNETH_REPO           GitHub "owner/repo" to download from
#   FENNETH_BASE_URL       Download from this URL (file:// works) instead of GitHub
set -eu

REPO="${FENNETH_REPO:-__REPO__}"
VERSION="${FENNETH_VERSION:-latest}"
BASE_URL="${FENNETH_BASE_URL:-}"

say() { printf '%s\n' "$*"; }
warn() { printf 'warning: %s\n' "$*" >&2; }
err() { printf 'error: %s\n' "$*" >&2; exit 1; }
have() { command -v "$1" >/dev/null 2>&1; }

fetch() { # fetch <url> <dest>
    if have curl; then
        curl -fsSL "$1" -o "$2"
    elif have wget; then
        wget -qO "$2" "$1"
    else
        err "curl or wget is required (Termux: pkg install curl)"
    fi
}

is_termux() {
    [ "$(uname -o 2>/dev/null || true)" = "Android" ] && [ -n "${PREFIX:-}" ]
}

# ---- platform ----------------------------------------------------------------

os="$(uname -s)"
arch="$(uname -m)"
target=""
case "$os" in
    Linux)
        case "$arch" in
            x86_64 | amd64) target="x86_64-unknown-linux-musl" ;;
            aarch64 | arm64) target="aarch64-unknown-linux-musl" ;;
        esac
        ;;
    Darwin)
        case "$arch" in
            arm64) target="aarch64-apple-darwin" ;;
            x86_64) target="x86_64-apple-darwin" ;;
        esac
        ;;
    *)
        err "unsupported OS '$os' (on Windows use install.ps1)"
        ;;
esac

# ---- install location -------------------------------------------------------------

if [ -n "${FENNETH_INSTALL:-}" ]; then
    root="$FENNETH_INSTALL"
elif is_termux; then
    root="$PREFIX"
else
    root="$HOME/.fenneth"
fi
bindir="$root/bin"

# ---- download URL --------------------------------------------------------------------

if [ -z "$BASE_URL" ]; then
    case "$REPO" in
        __REPO__*) err "this copy of the installer does not know its GitHub repo; set FENNETH_REPO=owner/repo" ;;
    esac
    if [ "$VERSION" = "latest" ]; then
        BASE_URL="https://github.com/$REPO/releases/latest/download"
    else
        BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
    fi
fi

# ---- fall back to building from source when there is no prebuilt binary ---------------------

if [ -z "$target" ]; then
    warn "no prebuilt Fenneth binary for $os/$arch"
    if have cargo && [ "$REPO" != "__REPO__" ]; then
        say "building from source with cargo..."
        if [ "$VERSION" = "latest" ]; then
            cargo install --locked --git "https://github.com/$REPO" fenneth --root "$root"
        else
            cargo install --locked --git "https://github.com/$REPO" --tag "$VERSION" fenneth --root "$root"
        fi
        target="source"
    else
        err "install Rust (Termux: pkg install rust) and run: cargo install --git https://github.com/$REPO fenneth"
    fi
fi

# ---- download, verify, install -------------------------------------------------------------

if [ "$target" != "source" ]; then
    asset="fenneth-$target.tar.gz"
    tmp="$(mktemp -d "${TMPDIR:-/tmp}/fenneth-install.XXXXXX")"
    trap 'rm -rf "$tmp"' EXIT INT TERM

    say "downloading $asset ($VERSION)..."
    fetch "$BASE_URL/$asset" "$tmp/$asset" || err "download failed: $BASE_URL/$asset"

    if fetch "$BASE_URL/$asset.sha256" "$tmp/$asset.sha256" 2>/dev/null; then
        expected="$(cut -d ' ' -f 1 "$tmp/$asset.sha256")"
        if have sha256sum; then
            actual="$(sha256sum "$tmp/$asset" | cut -d ' ' -f 1)"
        elif have shasum; then
            actual="$(shasum -a 256 "$tmp/$asset" | cut -d ' ' -f 1)"
        else
            actual=""
            warn "no sha256 tool found; skipping checksum verification"
        fi
        if [ -n "$actual" ] && [ "$actual" != "$expected" ]; then
            err "checksum mismatch for $asset (expected $expected, got $actual)"
        fi
    else
        warn "no checksum file published; skipping verification"
    fi

    have tar || err "tar is required"
    tar -xzf "$tmp/$asset" -C "$tmp"
    [ -f "$tmp/fenneth-$target/fenneth" ] || err "unexpected archive layout"

    mkdir -p "$bindir"
    cp "$tmp/fenneth-$target/fenneth" "$bindir/fenneth"
    chmod 755 "$bindir/fenneth"
fi

say "installed: $bindir/fenneth"
"$bindir/fenneth" --version

# ---- PATH ----------------------------------------------------------------------------------------

case ":$PATH:" in
    *":$bindir:"*) ;;
    *)
        line="export PATH=\"$bindir:\$PATH\""
        if [ "${FENNETH_NO_MODIFY_PATH:-0}" = "1" ]; then
            say "add this to your shell profile:  $line"
        else
            updated=""
            for rc in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.zshrc"; do
                # always create .profile, only touch the others if they exist
                if [ "$rc" = "$HOME/.profile" ] || [ -f "$rc" ]; then
                    if ! grep -qF "$bindir" "$rc" 2>/dev/null; then
                        printf '\n# Fenneth\n%s\n' "$line" >> "$rc"
                        updated="$updated $rc"
                    fi
                fi
            done
            [ -n "$updated" ] && say "added $bindir to PATH in:$updated"
            say "restart your shell, or run:  $line"
        fi
        ;;
esac

# ---- C compiler (Fenneth compiles to C, then needs gcc/clang to make a binary) ---------------------

if ! have gcc && ! have clang && ! have cc; then
    warn "no C compiler found; 'fenneth run/build' needs one ('fenneth emit/check' work without)"
    if is_termux; then
        say "  install one with:  pkg install clang"
    elif [ "$os" = "Darwin" ]; then
        say "  install one with:  xcode-select --install"
    else
        say "  install one with:  sudo apt install gcc   (or your distro's equivalent)"
    fi
fi

say "done. try:  echo 'fn main() -> i32 { print(\"hi\"); return 0; }' > hi.fen && fenneth hi.fen"
