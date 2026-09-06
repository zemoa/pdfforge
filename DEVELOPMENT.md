# Development guide

## Toolchain

- Node.js 24+ and Corepack; pnpm `11.25.0` is selected by `package.json`.
- Rust 1.98.1, installed with rustup (`rustup toolchain install 1.98.1`). CI pins this version; upgrade it deliberately in both workflows, then let CI create the corresponding Cargo cache.
- GitHub Actions supplies the Windows build environment. A local Windows developer needs Microsoft C++ Build Tools with **Desktop development with C++** and WebView2.
- Linux development needs the native packages required by Tauri. Ubuntu/Debian, Fedora and immutable Fedora/Bazzite instructions are below.

Enable pnpm once per machine:

```bash
corepack enable
```

Install JavaScript dependencies and run the application:

```bash
pnpm install --frozen-lockfile
pnpm tauri dev
```

On this Bazzite/Fedora OSTree machine, stage the Tauri build dependencies and reboot before the native build:

```bash
sudo rpm-ostree install webkit2gtk4.1-devel openssl-devel curl wget file libappindicator-gtk3-devel librsvg2-devel libxdo-devel gcc gcc-c++ make
sudo systemctl reboot
```

On Fedora, use `sudo dnf install` with the same package list. On Ubuntu/Debian, install `libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev`.

## Commands

| Command                                                            | Purpose                                                            |
| ------------------------------------------------------------------ | ------------------------------------------------------------------ |
| `pnpm dev`                                                         | Start Vite only.                                                   |
| `pnpm tauri dev`                                                   | Run the desktop application in development.                        |
| `pnpm build`                                                       | Type-check Vue and build the renderer.                             |
| `pnpm lint` / `pnpm lint:fix`                                      | Check / repair ESLint issues.                                      |
| `pnpm format:check` / `pnpm format`                                | Check / apply Prettier formatting.                                 |
| `pnpm test:unit`                                                   | Run unit tests; exits successfully when no domain tests exist yet. |
| `pnpm check`                                                       | Run the frontend quality gate.                                     |
| `cargo fmt --check --manifest-path src-tauri/Cargo.toml`           | Check Rust formatting.                                             |
| `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` | Lint Rust.                                                         |
| `pnpm tauri build`                                                 | Produce the local AppImage on Linux.                               |

`pnpm-workspace.yaml` explicitly permits esbuild's install script. Do not approve arbitrary dependency build scripts without reviewing why they are needed.

## Implementation workflow

1. Read `ARCHITECTURE.md` before changing source code.
2. Define the Rust domain and application use case first, then its unit tests.
3. Add a narrow Tauri command, capability permission, typed frontend application client and domain store intention only when the use case requires crossing IPC.
4. Build presentation components last. They only call store intentions and render state.
5. Run `pnpm check`, Rust formatting and Clippy before committing.

The component-to-backend shortcut is forbidden. See the normative layering rules in `ARCHITECTURE.md`.

## Updating the embedded PDFium runtime

FTR-002 embeds the PDFium 7881 x86_64 libraries under `src-tauri/resources/pdfium/`
for Linux and Windows. Keep their version compatible with the explicit
`pdfium-render` API feature in `src-tauri/Cargo.toml`; do not use a system PDFium
or download one at application runtime.

When updating PDFium, download the matching `pdfium-linux-x64.tgz` and
`pdfium-win-x64.tgz` artifacts from the tagged
[`bblanchon/pdfium-binaries`](https://github.com/bblanchon/pdfium-binaries)
release, retain their `LICENSE` and `licenses/` files, update the recorded
checksums below, regenerate `Cargo.lock`, and run the full quality gate.
The Linux library is copied into the AppImage by `tauri.conf.json`; the artifact
workflow copies the Windows DLL beside the executable before creating its ZIP.

Current PDFium 7881 library checksums:

- Linux `lib/libpdfium.so`: `f728930966f503652b92acc89b9374a2eeca00ce42e26dccd3e4b5c5161b2d64`;
- Windows `bin/pdfium.dll`: `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.

## i18n and appearance

Place product strings in both `src/i18n/locales/en.ts` and `src/i18n/locales/fr.ts`. Do not add untranslated UI text. The initial locale comes from the system and falls back to English. Theme mode is a UI preference (light, dark, system; default system) retained in localStorage.

## Application icon

`src-tauri/icons/app-icon.svg` is the canonical application-icon artwork. After changing it, regenerate all committed platform variants with:

```bash
pnpm tauri icon src-tauri/icons/app-icon.svg
```

## Versioning and commits

Use Conventional Commits, for example `feat(pdf): add merge use case`, `fix(ui): preserve selection`, `docs: clarify IPC boundary`, and `chore(deps): update tauri`. Do not create a changelog. Version releases by pushing a Git tag in the form `vX.Y.Z` after the project version has been updated.

Keep `pnpm-lock.yaml` and `src-tauri/Cargo.lock` committed. Direct JavaScript dependencies are exact. Update dependencies deliberately and verify the full quality gate.

## Signed update releases

Before publishing the first update-capable release, generate an Ed25519 key
pair and keep the private PEM outside the repository. Configure the base64
encoded 32-byte public key as the GitHub repository variable
`UPDATE_ED25519_PUBLIC_KEY`, and configure the matching PEM as the secret
`UPDATE_ED25519_PRIVATE_KEY`. The public key is compiled into release builds;
the private key is used only by the release workflow. Never commit private-key
material or a test key.

For an OpenSSL-generated private PEM, derive the repository variable with
`openssl pkey -in update-ed25519.pem -pubout -outform DER | tail -c 32 | base64 -w0`.

Prepare release `vX.Y.Z` with `pnpm release:prepare X.Y.Z`. It synchronizes
the version in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`
and `src-tauri/tauri.conf.json`, and creates the `release-notes/vX.Y.Z.json`
template. Fill its non-empty `fr` and `en` notes, then run
`pnpm release:publish X.Y.Z`. This validates the release, commits it as
`chore(release): prepare vX.Y.Z`, creates its tag and pushes the commit and tag
to `origin`. The workflow refuses a tag whose manifests or notes do not match.
It publishes a single Windows EXE and one Linux AppImage, each with `.sig` and
`.sha256` sidecars. Do not rename those assets manually: the update domain
matches their fixed names.

## CI, artifacts and signing

`.github/workflows/ci.yml` validates pull requests and pushes on Linux and Windows. `.github/workflows/artifacts.yml` runs only for `v*` tags, builds the Linux AppImage and a self-contained Windows executable, signs them for in-application verification, and publishes both files in a GitHub Release. The build artifacts remain available from the workflow run.

After pushing a version tag, find the distributable files in the repository’s
**Releases** page. Re-running the workflow for an existing tag replaces the
Release assets with the newly built files.

The CI and artifact workflows cache Cargo’s registry, Git dependencies and
`src-tauri/target` independently for each operating system and Rust compiler
version. Their exact cache key also includes `src-tauri/Cargo.lock`; on an
exact-key miss, the latest cache for the same operating system and compiler is
restored. Cargo validates the restored artifacts and rebuilds only the parts
incompatible with the lockfile. A Rust compiler upgrade still starts from an
empty Cargo cache.

Optional Windows signing uses these GitHub Actions secrets:

- `WINDOWS_CERTIFICATE_PFX`: base64-encoded PFX certificate;
- `WINDOWS_CERTIFICATE_PASSWORD`: its password;
- `WINDOWS_SIGN_TIMESTAMP_URL`: optional RFC 3161 timestamp service URL.

Without both certificate secrets, the portable executable is built unsigned. Never commit certificates, private keys or secrets.

## Documentation maintenance

All architecture decisions belong in `ARCHITECTURE.md`. All developer-facing operating instructions belong here. Update `AGENTS.md` only when the required AI workflow changes.
