# Rilis dan hosting docs

## Nomor versi

Rilis diberi nomor seperti Luau: **`0.xxx`** dengan tiga digit, misalnya `0.003`, `0.004`, `0.005`. Tag Git dan GitHub Release memakai nama persis itu (tanpa awalan `v`).

Nomor ditanamkan ke binary saat build lewat variabel lingkungan `FENNETH_VERSION`, sehingga `fenneth --version` mencetak `fenneth 0.003` dan workflow rilis memeriksanya. `version` di `Cargo.toml` hanya cadangan untuk build lokal; tidak perlu diubah saat merilis.

## Memublikasikan rilis

Workflow **Release** (`.github/workflows/release.yml`) membangun semua platform dan memublikasikan arsip, checksum, dan installer.

**Opsi 1: push tag**

```sh
git tag 0.003
git push origin 0.003
```

**Opsi 2: jalankan dari GitHub** (Actions → Release → Run workflow)

| Input | Arti |
|---|---|
| `version` | Kosongkan untuk memakai nomor berikutnya setelah tag `0.xxx` tertinggi (rilis pertama memakai `0.003`). |
| `publish` | Mati (default) = dry run: membangun semuanya, tanpa memublikasikan. Hidup = membuat tag dan GitHub Release. |

Workflow menolak versi yang bukan `0.xxx`, dan menolak membuat tag yang sudah ada.

## Yang dipublikasikan

| Aset | Isi |
|---|---|
| `fenneth-<target>.tar.gz` / `.zip` | Binary, `README.md`, `LICENSE`, `CHANGELOG.md` |
| `fenneth-<target>.<ext>.sha256` | Checksum SHA-256 |
| `install.sh`, `install.ps1` | Installer dengan nama repositori yang sudah ditanam |

Target: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-apple-darwin`.

## Continuous integration

Workflow **CI** berjalan pada setiap push ke `main` dan setiap pull request:

- build dan test di Linux, macOS, dan Windows;
- `cargo fmt --check`;
- membangun arsip rilis, memasangnya dengan skrip installer asli, lalu menjalankan binary yang terpasang;
- memeriksa bahwa setiap halaman docs bahasa Inggris punya padanan bahasa Indonesia, lalu membangun situs docs.

## Hosting docs ini di Vercel

Docs adalah situs [VitePress](https://vitepress.dev) di folder `docs/`, dengan bahasa Inggris di `/` dan bahasa Indonesia di `/id/`.

1. Di Vercel, pilih **Add New → Project** lalu import repositori.
2. Atur **Root Directory** ke `docs`.
3. Biarkan framework **VitePress**. `docs/vercel.json` sudah mengatur perintah install (`npm install`), perintah build (`npm run build`), dan folder output (`.vitepress/dist`).
4. Deploy. Setiap push ke `main` akan men-deploy ulang.
5. Ganti `OWNER/REPO` di `docs/.vitepress/config.mts` dengan repositori yang sebenarnya agar tautan GitHub dan "Edit halaman ini" berfungsi.

Jalankan secara lokal:

```sh
cd docs
npm install
npm run dev        # http://localhost:5173
```

## Menulis docs dalam dua bahasa

Setiap halaman Inggris `docs/<path>.md` harus punya halaman Indonesia `docs/id/<path>.md`. Jalankan `sh scripts/check-docs-i18n.sh` untuk memeriksa; CI juga menjalankannya dan gagal bila salah satu sisi hilang.
