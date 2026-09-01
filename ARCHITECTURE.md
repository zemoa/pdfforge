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
| CI                    | GitHub Actions validates pull requests and builds downloadable, non-release artifacts only for `vX.Y.Z` tags.                            |
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

## Dependency and quality policy

Direct JavaScript dependencies are exact versions in `package.json`; `pnpm-lock.yaml` is committed and authoritative. Rust resolves compatible current Tauri 2 crates into committed `Cargo.lock`. TypeScript is pinned to the latest version supported by `typescript-eslint` (currently 6.0.3), rather than an incompatible newer compiler. Upgrade deliberately, regenerate locks, run the full checks, and record a material architectural change here.

ESLint, Prettier, strict TypeScript, Rust formatting and Clippy are mandatory. Tests are unit tests only and cover business/application behaviour in the Rust backend (and a frontend domain store only when it contains behaviour). There are no end-to-end tests in this project at this stage.

## Architectural decision record

Material decisions are recorded in this document under the relevant section, with their date and rationale added when the project evolves. A change to platforms, distribution, security permissions, backend boundaries, state-management policy, persistence, external communication, telemetry, update policy, or a new major dependency is architectural and cannot be merged without updating this file.
