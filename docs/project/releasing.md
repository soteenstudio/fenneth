# Releases and docs hosting

## Version numbers

Releases are numbered like Luau: **`0.xxx`** with three digits, for example `0.003`, `0.004`, `0.005`. The Git tag and the GitHub Release have exactly that name (no `v` prefix).

The number is baked into the binary at build time through the `FENNETH_VERSION` environment variable, so `fenneth --version` prints `fenneth 0.003` and the release workflow checks it. The `version` in `Cargo.toml` is only the fallback for local builds; it does not need to change for a release.

## Publishing a release

The **Release** workflow (`.github/workflows/release.yml`) builds every platform and publishes the archives, checksums and installers.

**Option 1: push a tag**

```sh
git tag 0.003
git push origin 0.003
```

**Option 2: run it from GitHub** (Actions → Release → Run workflow)

| Input | Meaning |
|---|---|
| `version` | Leave empty to use the next number after the highest existing `0.xxx` tag (the first release uses `0.003`). |
| `publish` | Off (default) = dry run: builds everything, publishes nothing. On = creates the tag and the GitHub Release. |

The workflow refuses a version that is not `0.xxx`, and refuses to create a tag that already exists.

## What is published

| Asset | Content |
|---|---|
| `fenneth-<target>.tar.gz` / `.zip` | The binary, `README.md`, `LICENSE`, `CHANGELOG.md` |
| `fenneth-<target>.<ext>.sha256` | SHA-256 checksum |
| `install.sh`, `install.ps1` | Installers with the repository name filled in |

Targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-apple-darwin`.

## Continuous integration

The **CI** workflow runs on every push to `main` and every pull request:

- build and test on Linux, macOS and Windows;
- `cargo fmt --check`;
- build a release archive, install it with the real installer scripts, and run the installed binary;
- check that every English docs page has an Indonesian twin, and build the docs site.

## Hosting these docs on Vercel

The docs are a [VitePress](https://vitepress.dev) site in the `docs/` folder, with English at `/` and Indonesian at `/id/`.

1. In Vercel, choose **Add New → Project** and import the repository.
2. Set **Root Directory** to `docs`.
3. Leave the framework as **VitePress**. `docs/vercel.json` already sets the install command (`npm install`), the build command (`npm run build`) and the output folder (`.vitepress/dist`).
4. Deploy. Every push to `main` redeploys.
5. Replace `OWNER/REPO` in `docs/.vitepress/config.mts` with the real repository so the GitHub and "Edit this page" links work.

Run it locally:

```sh
cd docs
npm install
npm run dev        # http://localhost:5173
```

## Writing docs in two languages

Every English page `docs/<path>.md` must have an Indonesian page `docs/id/<path>.md`. Run `sh scripts/check-docs-i18n.sh` to check; CI runs it too and fails when one side is missing.
