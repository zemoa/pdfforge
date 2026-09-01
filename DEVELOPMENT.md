# Development guide

## Toolchain

- Node.js 24+ and Corepack; pnpm `11.25.0` is selected by `package.json`.
- Rust stable, installed with rustup.
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

## i18n and appearance

Place product strings in both `src/i18n/locales/en.ts` and `src/i18n/locales/fr.ts`. Do not add untranslated UI text. The initial locale comes from the system and falls back to English. Theme mode is a UI preference (light, dark, system; default system) retained in localStorage.

## Versioning and commits

Use Conventional Commits, for example `feat(pdf): add merge use case`, `fix(ui): preserve selection`, `docs: clarify IPC boundary`, and `chore(deps): update tauri`. Do not create a changelog. Version releases by pushing a Git tag in the form `vX.Y.Z` after the project version has been updated.

Keep `pnpm-lock.yaml` and `src-tauri/Cargo.lock` committed. Direct JavaScript dependencies are exact. Update dependencies deliberately and verify the full quality gate.

## CI, artifacts and signing

`.github/workflows/ci.yml` validates pull requests and pushes on Linux and Windows. `.github/workflows/artifacts.yml` runs only for `v*` tags, builds the Linux AppImage and the ZIP containing the Windows executable, then publishes both files in a GitHub Release with automatically generated notes. The build artifacts remain available from the workflow run.

After pushing a version tag, find the distributable files in the repository’s
**Releases** page. Re-running the workflow for an existing tag replaces the
Release assets with the newly built files.

Optional Windows signing uses these GitHub Actions secrets:

- `WINDOWS_CERTIFICATE_PFX`: base64-encoded PFX certificate;
- `WINDOWS_CERTIFICATE_PASSWORD`: its password;
- `WINDOWS_SIGN_TIMESTAMP_URL`: optional RFC 3161 timestamp service URL.

Without both certificate secrets, the portable executable is built unsigned. Never commit certificates, private keys or secrets.

## Documentation maintenance

All architecture decisions belong in `ARCHITECTURE.md`. All developer-facing operating instructions belong here. Update `AGENTS.md` only when the required AI workflow changes.
