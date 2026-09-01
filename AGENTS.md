# Guidance for coding agents

Before changing this repository, read `ARCHITECTURE.md` and `DEVELOPMENT.md` completely. They are normative.

- Keep all business logic in the Rust backend. Components and pages never call Tauri APIs or the backend directly.
- Follow the prescribed path: component/page → domain store intention → typed frontend client → narrow Tauri command → Rust application/domain service.
- A Pinia store represents a business domain and exposes intentions/queries, never generic setters. Use composables for UI-only state.
- Respect SOLID, strict TypeScript, Naive UI, English/French translations and the least-privilege Tauri capability/CSP policy.
- Add unit tests for business/application logic only; do not introduce E2E testing without an explicit architectural change.
- Run the relevant checks in `DEVELOPMENT.md`. Keep lockfiles committed.
- Record every architectural choice in `ARCHITECTURE.md` and every new development procedure in `DEVELOPMENT.md`.
- Use Conventional Commits. Do not add a changelog, telemetry, automatic updates, or installer packaging without an explicit documented decision.
