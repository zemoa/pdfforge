---
name: pdfforge-release
description: Assess, propose, and prepare a PDFForge version release with bilingual notes. Use when asked to choose, propose, prepare, or draft a PDFForge release; it never publishes or pushes one.
---

# PDFForge Release Preparation

Assess and prepare the requested PDFForge release without publishing it. The user always owns the final version choice.

## Scope and source of truth

- Work in the PDFForge repository. Read the **Signed update releases** section of `DEVELOPMENT.md` and inspect `scripts/release.mjs` before acting, so the local procedure remains authoritative.
- This skill may run `pnpm release:prepare X.Y.Z` and edit only the generated `release-notes/vX.Y.Z.json` as part of preparing a version the user has explicitly confirmed.
- It must never run `pnpm release:publish`, create a Git tag, push a commit or tag, create/edit a GitHub Release, or rename release assets. Those are a separate, explicit publishing action.

## Preparation workflow

1. Inspect `git status --short`, the current versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`, and the changes since the previous reachable version tag. Report unrelated existing changes; do not overwrite them without the user's direction.
2. Categorize the evidence and propose one next version from the current version:
   - Increment `X` and reset `Y.Z` to `0.0` for an added or materially modified major feature.
   - Otherwise, increment `Y` and reset `Z` to `0` for the addition of one or more features.
   - Otherwise, increment `Z` for corrections only.
   - When the release contains several categories, use the highest applicable increment: major, then minor, then patch.
   Explain the evidence and classification behind the recommendation. Do not invent features, fixes, or user impact.
3. The recommendation is advisory only. Do not run `pnpm release:prepare`, edit a version manifest, or create release notes until the user explicitly confirms the exact final `X.Y.Z` version. A version mentioned as an example or a possible target is not confirmation; an explicit instruction to prepare that exact version is.
4. After confirmation, check whether `release-notes/vX.Y.Z.json` already exists, then run `pnpm release:prepare X.Y.Z`. It synchronizes the three manifests and creates the notes template when needed.
5. Draft concise, factual, user-facing notes in both `fr` and `en` in `release-notes/vX.Y.Z.json`. Use the repository history and changed files as evidence. If there are no suitable changes or their user impact is unclear, ask the user for the missing information and leave the notes pending rather than fabricating them.
6. When both notes are available, validate that all three manifests contain exactly `X.Y.Z`, the JSON is valid, its `version` is exactly `X.Y.Z`, and its `fr` and `en` strings are non-empty after trimming. Show the notes and the resulting working-tree changes for review.

## Handoff

State clearly that the release is prepared but not published. Give the exact next command only after user approval to publish:

```bash
pnpm release:publish X.Y.Z
```

This command commits, tags, and pushes; never execute it under this skill unless the user makes a new explicit request to publish.
