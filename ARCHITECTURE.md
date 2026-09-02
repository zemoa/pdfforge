# PDFForge architecture

## Purpose and scope

PDFForge is a portable desktop application owned by Zemoa. This repository currently contains only its technical foundation; product behaviour is intentionally not specified here. The application identifier is `com.zemoa.pdfforge`, the product and window name are `PDFForge`, and the licence is `AGPL-3.0-or-later`.

## Platform and delivery decisions

| Concern               | Decision                                                                                                                                 |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Desktop framework     | Latest stable Tauri 2, with a Rust backend and a Vue 3 TypeScript renderer.                                                              |
| Supported targets     | Linux x86_64 and Windows x86_64.                                                                                                         |
| Linux delivery        | AppImage only. It is portable; a host may need FUSE to mount it. Compatibility is targeted at Ubuntu, Debian and Fedora.                 |
| Windows delivery      | A ZIP containing `PDFForge.exe`; it is not an installer. WebView2 is relied upon for now.                                                |
| CI                    | GitHub Actions validates pull requests and publishes a GitHub Release with the Linux AppImage and Windows ZIP for `vX.Y.Z` tags.         |
| Updates and telemetry | Neither automatic updates nor telemetry are included.                                                                                    |
| Windows signing       | Optional in CI when a PFX certificate and password secrets are supplied; otherwise the artifact is unsigned and may trigger SmartScreen. |
| Branding              | The generated Tauri icon is temporary until Zemoa supplies the PDFForge visual identity.                                                 |

## Layering and dependency direction

The renderer is a presentation layer only. Product and business logic live in Rust. Dependencies point inward:

```text
Vue component/page -> domain Pinia store -> frontend application client -> Tauri command adapter -> Rust application service -> Rust domain
                                                                                                      -> Rust infrastructure
```

1. A Vue component or page **must never** import `@tauri-apps/api`, call `invoke`, or otherwise contact the backend directly.
2. A Tauri command is a thin input/output adapter. It validates and converts DTOs, calls one Rust application service, and returns a DTO. It contains no business rules.
3. Rust is split by business domain. For each domain, use `domain` (entities, value objects, invariants), `application` (use cases and ports), `infrastructure` (filesystem, parsers, persistence and other adapters), and `presentation`/`commands` (Tauri adapters).
4. Domain and application Rust modules do not depend on Tauri, Vue, filesystem implementations, or other infrastructure details. Inject ports at the composition root.
5. The frontend client is the only frontend module allowed to import Tauri APIs. It gives commands typed request/response DTOs and maps technical errors into application-level errors.
6. Pinia stores represent one named business domain only. They expose intentions/use cases, queries and derived state—never public generic setters. A component asks a store to perform an intention; it does not mutate business state itself.
7. UI-only concerns, such as the current theme and display locale, use composables rather than a domain store. The theme preference is the sole localStorage use in this baseline and is not business data.
8. New cross-domain communication happens through explicit application interfaces or domain events, not imports between unrelated stores or UI components.

These rules implement SOLID: one reason to change per module, domain contracts rather than concrete infrastructure, and small focused interfaces. Prefer immutable data transfer objects and exhaustive TypeScript/Rust types at boundaries.

## Frontend

`src/` is organized by responsibility:

- `pages/` compose route-level UI; `components/` will hold reusable UI only.
- `stores/<domain>/` will contain Pinia stores when a business domain exists.
- `application/` will contain the typed frontend clients used by stores.
- `composables/` contains UI-only behaviour.
- `i18n/` contains English and French translations. Startup detects the system language and falls back to English.
- `router/` owns routing. Hash history is used so packaged local assets need no server rewrite support.

Vue uses Composition API and strict TypeScript. Naive UI is the only component library. The renderer supports light, dark and system modes; system is the default. No product store has been created because no product domain has been defined.

## Backend and IPC

`src-tauri/` is the application boundary. No command is exposed in the initial state. Adding a command requires all of the following: a business-domain owner, a typed DTO, a least-privilege capability entry, a frontend application client, a domain store intention, unit tests of the application/domain behaviour, and an architecture-document update.

The default capability grants only `core:default`; no filesystem, shell, HTTP, opener, updater, clipboard or other plugin permission is pre-authorized. The CSP allows only local application assets and the Tauri IPC transport. Any new permission or CSP source is an architectural decision and must be documented here before implementation.

### PDF merge domain (2026-09-01)

FTR-001 introduces the `merge` Rust business domain with domain, application,
infrastructure and presentation/command modules. Its application layer depends
on ports for source inspection, output reservation, PDF merging and result
opening. `lopdf` is the initial local implementation of the merge-engine port;
it is deliberately replaceable without changing the IPC DTOs or Pinia store.
The UI warns before confirmation when source catalog structures (outlines,
forms, tagged structure or named destinations) cannot be guaranteed by this
engine. PDF processing stays local and no source path is persisted.

The main-window capability additionally grants `dialog:allow-open` for the
native file and folder selector and `core:window:allow-destroy` so Tauri can
complete a user-requested window close after the merge close guard has allowed
it. Opening a successful output happens in the Rust backend through the opener
plugin, never as a renderer-exposed broad path-opening permission.

### PDF split domain and embedded renderer (2026-09-01)

FTR-002 introduces the independent `split` Rust business domain. Its application
layer owns source inspection, page-selection validation, output-batch
reservation, thumbnail rendering and page extraction ports. The frontend reaches
it only through the `split` Pinia store and typed client. A split has one source;
groups are page sets with no overlap, and all temporary reservations are removed
on cancellation or failure.

`pdfium-render` backed by the checked-in PDFium 7881 Linux and Windows x86_64
libraries is the replaceable local implementation for thumbnails and page copy.
The Linux library and notices are AppImage resources; the Windows portable ZIP
ships the matching DLL beside `PDFForge.exe`. Neither target needs a system
library or network connection at runtime. PDFium access is serialized inside the
infrastructure adapter. Successful output opening remains a backend opener
action; the renderer receives no path-opening or filesystem permission.

### PDF text-redaction preparation (2026-09-02)

FTR-003 introduces the independent `redaction` Rust business domain for
preparing text redactions. Its source inspector accepts one local readable PDF
without password prompting, while its PDFium-backed page renderer emits only
the current rasterized page and normalized word bounds. The renderer sends
selection intentions through the `redaction` Pinia store and keeps source and
selection data in memory only; no path, page image or selection survives an
application restart. The main-window capability grants only the two narrow
inspection and page-rendering commands, alongside the existing native file
dialog permission. Rectangle preparation and irreversible PDF rewriting remain
separate future domain use cases.

## Dependency and quality policy

Direct JavaScript dependencies are exact versions in `package.json`; `pnpm-lock.yaml` is committed and authoritative. Rust resolves compatible current Tauri 2 crates into committed `Cargo.lock`. TypeScript is pinned to the latest version supported by `typescript-eslint` (currently 6.0.3), rather than an incompatible newer compiler. Upgrade deliberately, regenerate locks, run the full checks, and record a material architectural change here.

ESLint, Prettier, strict TypeScript, Rust formatting and Clippy are mandatory. Tests are unit tests only and cover business/application behaviour in the Rust backend (and a frontend domain store only when it contains behaviour). There are no end-to-end tests in this project at this stage.

## Architectural decision record

Material decisions are recorded in this document under the relevant section, with their date and rationale added when the project evolves. A change to platforms, distribution, security permissions, backend boundaries, state-management policy, persistence, external communication, telemetry, update policy, or a new major dependency is architectural and cannot be merged without updating this file.

### GitHub Releases for version tags (2026-09-01)

Each pushed `vX.Y.Z` tag now publishes a GitHub Release after the Linux and
Windows portable builds succeed. The Release receives the generated notes and
contains the AppImage and the ZIP with `PDFForge.exe`; workflow artifacts are
kept as the build-job outputs as well.
