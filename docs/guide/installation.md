# Installation

## Install a release

Replace `OWNER/REPO` with the GitHub repository that publishes the releases.

**Linux, macOS and Termux (Android)**

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

Check it works:

```sh
fenneth --version
```

### What the installer does

1. Detects your OS and CPU and downloads the matching archive from the GitHub Release.
2. Verifies the SHA-256 checksum.
3. Copies the binary to its install folder and adds that folder to your `PATH`.
4. Warns you if no C compiler is installed.

| Platform | Default install folder |
|---|---|
| Linux, macOS | `~/.fenneth/bin` |
| Termux | `$PREFIX/bin` |
| Windows | `%LOCALAPPDATA%\Fenneth\bin` |

### Prebuilt binaries

| Platform | Archive |
|---|---|
| Linux x86_64 (static, musl) | `fenneth-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 (static, musl, also Termux) | `fenneth-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `fenneth-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `fenneth-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `fenneth-x86_64-pc-windows-msvc.zip` |

Every archive has a matching `.sha256` file. Where no prebuilt binary exists (for example 32-bit Termux), the Linux/macOS installer falls back to `cargo install` if Rust is installed.

### Installer variables

| Variable | Purpose |
|---|---|
| `FENNETH_VERSION` | Install a specific release, for example `0.001`. |
| `FENNETH_INSTALL` | Install folder; the binary goes in `<folder>/bin`. |
| `FENNETH_NO_MODIFY_PATH=1` | Do not touch shell profiles or the user `PATH`. |
| `FENNETH_REPO` | `owner/repo` to download from, if your copy of the installer does not have it baked in. |
| `FENNETH_BASE_URL` | Download from this URL (or local folder) instead of GitHub. |

Example: install release `0.003` into a custom folder.

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh \
  | FENNETH_VERSION=0.003 FENNETH_INSTALL=$HOME/tools/fenneth sh
```

## Install a C compiler

`fenneth run` and `fenneth build` generate C and then call a C compiler. `fenneth emit` and `fenneth check` do not need one.

| Platform | How to install |
|---|---|
| Termux | `pkg install clang` (and `pkg install curl` for the installer) |
| Debian / Ubuntu | `sudo apt install gcc` |
| macOS | `xcode-select --install` |
| Windows | `winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT` (MinGW-w64 gcc), or MSYS2 |

The compiler is looked up in this order: `--cc`, `$FENNETH_CC`, `gcc`, `clang`, `cc`.

## Build from source

You need a recent stable Rust toolchain.

```sh
cargo install --git https://github.com/OWNER/REPO fenneth
# or, from a local checkout:
cargo build --release        # binary: target/release/fenneth
```

On Termux: `pkg install rust clang`, then run the `cargo install` command above.

A build from source reports the version from `Cargo.toml`. Release builds report the release number (`0.xxx`); see [Releases](/project/releasing).

## Uninstall

Delete the install folder (for example `~/.fenneth`) and remove the `# Fenneth` lines the installer added to `~/.profile`, `~/.bashrc` or `~/.zshrc`. On Windows, delete `%LOCALAPPDATA%\Fenneth` and remove its `bin` folder from your user `PATH`.
