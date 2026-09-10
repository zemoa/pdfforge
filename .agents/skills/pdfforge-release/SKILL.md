---
name: pdfforge-release
description: Assess, propose, and prepare a PDFForge version release with bilingual notes. Use when asked to choose, propose, prepare, or draft a PDFForge release; it never publishes or pushes one.
---

# PDFForge Release Preparation

Assess and prepare the requested PDFForge release without publishing it. This
is an interactive workflow: the user always approves both the exact version and
the complete bilingual release notes before the skill changes any release file.

## Model and delegation

Use Luna (`gpt-5.6-luna`) for release assessment, bilingual notes and approved
preparation. If already running as Luna, execute the workflow directly and do
not delegate it again. Otherwise, delegate each stage to one Luna subagent with
`collaboration.spawn_agent`, setting `model: "gpt-5.6-luna"` and
`fork_turns: "none"`. Include the repository path, this skill's path, the user's
request and any exact version or notes already approved in the task message.
While Luna works, the parent checks the applicable release procedure and reviews
the returned evidence; it relays proposals and collects the required approvals.
Resume the same Luna agent with `collaboration.followup_task` for later stages,
passing the user's exact approvals or revisions. Do not let the agent proceed
past an approval boundary without that approval. If Luna or delegation is
unavailable, report the limitation instead of silently using another model.

## Scope and source of truth

- Work in the PDFForge repository. Read the **Signed update releases** section of `DEVELOPMENT.md` and inspect `scripts/release.mjs` before acting, so the local procedure remains authoritative.
- This skill may run `pnpm release:prepare X.Y.Z` and edit only the generated `release-notes/vX.Y.Z.json` after the user has explicitly approved that exact version and the complete bilingual notes.
- It must never run `pnpm release:publish`, create a Git tag, push a commit or tag, create/edit a GitHub Release, or rename release assets. Those are a separate, explicit publishing action.

## Interactive preparation workflow

1. Inspect `git status --short`, the current versions in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`, and the changes since the previous reachable version tag. Report unrelated existing changes; do not overwrite them without the user's direction.
2. Categorize the evidence and propose one next version from the current version:
   - Increment `X` and reset `Y.Z` to `0.0` for an added or materially modified major feature.
   - Otherwise, increment `Y` and reset `Z` to `0` for the addition of one or more features.
   - Otherwise, increment `Z` for corrections only.
   - When the release contains several categories, use the highest applicable increment: major, then minor, then patch.
   Explain the evidence and classification behind the recommendation. Do not invent features, fixes, or user impact.
3. Present the proposed exact `X.Y.Z` version and **pause for the user's explicit validation**. Do not run `pnpm release:prepare`, edit a version manifest, or create release notes at this stage. A version mentioned as an example or a possible target is not confirmation. The user must explicitly approve the exact version (for example, “I approve version X.Y.Z” or “Prépare la version X.Y.Z”).
4. Once the exact version is approved, draft concise, factual, user-facing notes in both `fr` and `en` from the repository history and changed files. Show the complete proposed notes in the conversation and **pause for the user's explicit validation of their content**. Do not change any release file yet. If there are no suitable changes or their user impact is unclear, ask the user for the missing information and leave the notes pending rather than fabricating them.
5. Apply the release changes only after the user explicitly approves the exact bilingual notes. If the user requests a note change, revise the draft, show both complete notes again, and wait for a new explicit validation. If the user changes the version, return to step 3: validate the new exact version, then draft and validate its notes.
6. After both approvals, check whether `release-notes/vX.Y.Z.json` already exists, then run `pnpm release:prepare X.Y.Z`. It synchronizes the three manifests and creates the notes template when needed. Write exactly the approved `fr` and `en` notes in that JSON file.
7. Validate that all three manifests contain exactly `X.Y.Z`, the JSON is valid, its `version` is exactly `X.Y.Z`, and its `fr` and `en` strings are non-empty after trimming. Show the approved notes and the resulting working-tree changes. State clearly that the release is prepared but not published.

## Handoff

State clearly that the release is prepared but not published. Give the exact next command only after user approval to publish:

```bash
pnpm release:publish X.Y.Z
```

This command commits, tags, and pushes; never execute it under this skill unless the user makes a new explicit request to publish.
