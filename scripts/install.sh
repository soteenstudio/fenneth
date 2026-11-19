#!/bin/sh
# Fenneth installer for Linux, macOS, Termux (Android) and Windows shells that
# provide a POSIX environment (Git Bash, MSYS2, Cygwin). On Windows it installs
# the prebuilt fenneth.exe; install.ps1 is the native alternative.
#
#   curl -fsSL https://github.com/soteenstudio/fenneth/releases/latest/download/install.sh | sh
#
# Environment variables:
#   FENNETH_VERSION        Release tag to install, e.g. 0.003 (default: latest)
#   FENNETH_INSTALL        Install root; the binary goes to $FENNETH_INSTALL/bin
#                          (default: $HOME/.fenneth, or $PREFIX on Termux)
#   FENNETH_NO_MODIFY_PATH Set to 1 to leave your shell profiles untouched
#   FENNETH_REPO           GitHub "owner/repo" to download from
#   FENNETH_BASE_URL       Download from this URL (file:// works) instead of GitHub
set -eu

REPO="${FENNETH_REPO:-soteenstudio/fenneth}"
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

# Windows shells with a POSIX layer: Git Bash / MSYS2 (MINGW*, MSYS*) and Cygwin.
is_windows() {
    case "$(uname -s)" in
        MINGW* | MSYS* | CYGWIN* | Windows_NT) return 0 ;;
    esac
    return 1
}

# -- to unix path (cygpath, when present) -----------------------------------------
winpath() { # winpath <path>
    if have cygpath; then cygpath -u "$1"; else printf '%s\n' "$1"; fi
}

# ---- platform ----------------------------------------------------------------

os="$(uname -s)"
arch="$(uname -m)"
target=""
exe="fenneth"
archive_ext="tar.gz"

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
    MINGW* | MSYS* | CYGWIN* | Windows_NT)
        # Windows on ARM64 runs the x64 build through emulation.
        target="x86_64-pc-windows-msvc"
        exe="fenneth.exe"
        archive_ext="zip"
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
elif is_windows && [ -n "${LOCALAPPDATA:-}" ]; then
    # same place install.ps1 uses, so both installers share one binary
    root="$(winpath "$LOCALAPPDATA")/Fenneth"
else
    root="$HOME/.fenneth"
fi
bindir="$root/bin"

# ---- download URL --------------------------------------------------------------------

if [ -z "$BASE_URL" ]; then
    [ -n "$REPO" ] || err "FENNETH_REPO is empty; set it to owner/repo"
    if [ "$VERSION" = "latest" ]; then
        BASE_URL="https://github.com/$REPO/releases/latest/download"
    else
        BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
    fi
fi

# ---- fall back to building from source when there is no prebuilt binary ---------------------

if [ -z "$target" ]; then
    warn "no prebuilt Fenneth binary for $os/$arch"
    if have cargo; then
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
    asset="fenneth-$target.$archive_ext"
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

    if [ "$archive_ext" = "zip" ]; then
        if have unzip; then
            unzip -q "$tmp/$asset" -d "$tmp" || err "failed to extract $asset"
        elif have bsdtar; then
            bsdtar -xf "$tmp/$asset" -C "$tmp" || err "failed to extract $asset"
        elif have powershell.exe; then
            powershell.exe -NoProfile -NonInteractive -Command \
                "Expand-Archive -LiteralPath '$(winpath "$tmp/$asset")' -DestinationPath '$(winpath "$tmp")' -Force" \
                || err "failed to extract $asset"
        else
            err "unzip is required to extract $asset"
        fi
    else
        have tar || err "tar is required"
        tar -xzf "$tmp/$asset" -C "$tmp"
    fi
    [ -f "$tmp/fenneth-$target/$exe" ] || err "unexpected archive layout"

    mkdir -p "$bindir"
    cp "$tmp/fenneth-$target/$exe" "$bindir/$exe"
    chmod 755 "$bindir/$exe" 2>/dev/null || true
fi

say "installed: $bindir/$exe"
"$bindir/$exe" --version

# ---- PATH ----------------------------------------------------------------------------------------

if case ":$PATH:" in *":$bindir:"*) false ;; *) true ;; esac; then
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

        # On Windows also register the directory in the user PATH, so cmd.exe and
        # PowerShell can find fenneth.exe too (not just Git Bash / MSYS2 / Cygwin).
        if is_windows && have powershell.exe; then
            wbindir="$(cygpath -w "$bindir" 2>/dev/null || printf '%s' "$bindir")"
            wbindir="$(printf '%s' "$wbindir" | sed "s/'/''/g")"
            powershell.exe -NoProfile -NonInteractive -Command "
                \$dir = '$wbindir'
                \$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
                \$entries = @(\$userPath -split ';' | Where-Object { \$_ })
                if (\$entries -notcontains \$dir) {
                    \$newPath = (@(\$entries) + \$dir) -join ';'
                    [Environment]::SetEnvironmentVariable('Path', \$newPath, 'User')
                    Write-Host \"added \$dir to your user PATH (open a new terminal to pick it up)\"
                }
            " || warn "could not update the user PATH; add $wbindir manually"
        fi
    fi
fi

# ---- C compiler (Fenneth compiles to C, then needs gcc/clang to make a binary) ---------------------

if ! have gcc && ! have clang && ! have cc; then
    warn "no C compiler found; 'fenneth run/build' needs one ('fenneth emit/check' work without)"
    if is_termux; then
        say "  install one with:  pkg install clang"
    elif [ "$os" = "Darwin" ]; then
        say "  install one with:  xcode-select --install"
    elif is_windows; then
        say "  easiest option (MinGW-w64 gcc, self-contained):"
        say "    winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT"
        say "  or MSYS2 (https://www.msys2.org), then:  pacman -S mingw-w64-ucrt-x86_64-gcc"
        say "  (Git Bash alone does not ship a C compiler)"
    else
        say "  install one with:  sudo apt install gcc   (or your distro's equivalent)"
    fi
fi

say 'done. try:  echo "fn main() -> i32 { print(\"hi\"); return 0; }" > hi.fen && fenneth hi.fen'

