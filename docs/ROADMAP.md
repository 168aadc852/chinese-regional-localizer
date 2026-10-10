# Roadmap

Updated: 2026-10-10. This consolidated direction supersedes the earlier roadmap.
Planned capabilities must not be presented as implemented.

## Current priority and next phase

Phase 3J / Issue #38 desktop settings persistence is complete.

Issue #46's approved standalone profile/model/store foundation is merged; see
`CONTEXT_PROFILES.md`. Issue #47's core terminology selection is merged; see
`CONTEXT_SELECTION.md`. Issue #48's core candidate/one-time review layer is merged;
see `ALTERNATIVE_TERMS.md`. Issue #49's approved My Dictionaries/private preferences
core is implemented in its focused PR, pending CI/review/merge; see `MY_DICTIONARIES.md`.
Desktop choice/context/remember and dictionary-manager controls remain future work.

Review `HANCONTEXT_ALPHA_MVP.md` and the scoped work packages:

1. Context profile/data model foundation (#46); merged.
2. Context-aware deterministic terminology selection (#47); merged.
3. Core alternative terms and occurrence-safe one-time review (#48); merged.
4. Context-aware My Terms / remembered preferences (#49); review its focused PR.
5. Alpha desktop UX and plain-language explanations.
6. Limited Alpha packaging and onboarding.

Each requires separate approval and a focused PR. Preserve Runtime API v1 unless
a specific issue approves a change; preserve licence gates and offline operation.
Issues #50–#51 need separate design/implementation approval; do not start them automatically.

## Consolidated delivery direction

Desktop Alpha → product CLI → MCP Server → production Windows/macOS packaging/signing
→ editor/plugin integrations → optional AI assistance.

HanContext Core / Runtime serves Desktop, CLI and MCP: same rules, same data, same
result. The Rust demo CLI already uses Runtime API v1; product CLI batch, scripting,
automation and CI workflows are future work. A future MCP server may expose localization,
terminology, alternatives and explanations through that same runtime.

Release hosting/CDN, retry/resume and key operations remain separate follow-ups.
Additional regional routes, governed context/source coverage, file workflows and
mobile delivery need their own issues. Use Chinese Mainland, Hong Kong and Taiwan
in future copy. Precise implemented status is in `../PROJECT_STATE.md` and `../CHANGELOG.md`.

## Implemented foundations

Phase 0 has a first-pass source-whitelist conclusion with unresolved rights kept
non-ingest. Phases 1–2 built the SQLite proof of concept, Python reference engine,
hardening, private user control and evaluation. Phases 3A–3J added Rust/runtime parity,
the minimal Tauri desktop, validated database selection, immutable packages/rollback,
signing, authenticated network updates, manual desktop update controls and safe
versioned desktop settings persistence. OpenCC remains partial/reference. Detailed
phase history belongs in `../CHANGELOG.md` and `history/PROJECT_HISTORY.md`, rather
than implying every older roadmap item shipped.
