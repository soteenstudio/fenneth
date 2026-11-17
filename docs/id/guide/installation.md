# Instalasi

## Pasang dari rilis

Ganti `OWNER/REPO` dengan repositori GitHub yang memublikasikan rilis.

**Linux, macOS, dan Termux (Android)**

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh | sh
```

**Windows (PowerShell)**

```powershell
irm https://github.com/OWNER/REPO/releases/latest/download/install.ps1 | iex
```

Pastikan berhasil:

```sh
fenneth --version
```

### Yang dilakukan installer

1. Mendeteksi OS dan CPU Anda, lalu mengunduh arsip yang sesuai dari GitHub Release.
2. Memverifikasi checksum SHA-256.
3. Menyalin binary ke folder instalasi dan menambahkan folder itu ke `PATH`.
4. Memberi peringatan bila compiler C belum terpasang.

| Platform | Folder instalasi default |
|---|---|
| Linux, macOS | `~/.fenneth/bin` |
| Termux | `$PREFIX/bin` |
| Windows | `%LOCALAPPDATA%\Fenneth\bin` |

### Binary siap pakai

| Platform | Arsip |
|---|---|
| Linux x86_64 (statis, musl) | `fenneth-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 (statis, musl, termasuk Termux) | `fenneth-aarch64-unknown-linux-musl.tar.gz` |
| macOS Apple Silicon | `fenneth-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `fenneth-x86_64-apple-darwin.tar.gz` |
| Windows x86_64 | `fenneth-x86_64-pc-windows-msvc.zip` |

Setiap arsip punya file `.sha256`. Bila tidak ada binary siap pakai (misalnya Termux 32-bit), installer Linux/macOS beralih ke `cargo install` jika Rust terpasang.

### Variabel installer

| Variabel | Fungsi |
|---|---|
| `FENNETH_VERSION` | Pasang rilis tertentu, mis. `0.003` (default: terbaru). |
| `FENNETH_INSTALL` | Folder instalasi; binary berada di `<folder>/bin`. |
| `FENNETH_NO_MODIFY_PATH=1` | Jangan ubah profil shell atau `PATH` pengguna. |
| `FENNETH_REPO` | `owner/repo` sumber unduhan, jika installer Anda belum menanamkannya. |
| `FENNETH_BASE_URL` | Unduh dari URL (atau folder lokal) ini, bukan dari GitHub. |

Contoh: pasang rilis `0.003` ke folder sendiri.

```sh
curl -fsSL https://github.com/OWNER/REPO/releases/latest/download/install.sh \
  | FENNETH_VERSION=0.003 FENNETH_INSTALL=$HOME/tools/fenneth sh
```

## Pasang compiler C

`fenneth run` dan `fenneth build` menghasilkan C lalu memanggil compiler C. `fenneth emit` dan `fenneth check` tidak membutuhkannya.

| Platform | Cara memasang |
|---|---|
| Termux | `pkg install clang` (dan `pkg install curl` untuk installer) |
| Debian / Ubuntu | `sudo apt install gcc` |
| macOS | `xcode-select --install` |
| Windows | `winget install -e --id BrechtSanders.WinLibs.POSIX.UCRT` (MinGW-w64 gcc), atau MSYS2 |

Compiler dicari berurutan: `--cc`, `$FENNETH_CC`, `gcc`, `clang`, `cc`.

## Build dari source

Anda membutuhkan toolchain Rust stable yang cukup baru.

```sh
cargo install --git https://github.com/OWNER/REPO fenneth
# atau, dari checkout lokal:
cargo build --release        # binary: target/release/fenneth
```

Di Termux: `pkg install rust clang`, lalu jalankan perintah `cargo install` di atas.

Build dari source melaporkan versi dari `Cargo.toml`. Build rilis melaporkan nomor rilis (`0.xxx`); lihat [Rilis](/id/project/releasing).

## Mencopot

Hapus folder instalasi (mis. `~/.fenneth`) dan hapus baris `# Fenneth` yang ditambahkan installer ke `~/.profile`, `~/.bashrc`, atau `~/.zshrc`. Di Windows, hapus `%LOCALAPPDATA%\Fenneth` dan buang folder `bin`-nya dari `PATH` pengguna.
