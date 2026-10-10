# D-026 — Desktop presentation stays separate from localization

Date: 2026-10-10

Issue #60 implements the approved #50 multi-language, theme/accessibility and
thin-bridge contracts as a narrow static-shell foundation.

Use one human-editable UTF-8 CSV table, stable semantic keys and four independent
UI catalogs. Generate deterministic JSON plus a classic-script offline bundle;
validate completeness/placeholders/references/drift in CI and every Cargo desktop
build. No framework or runtime Chinese script conversion is introduced. The
existing repository Python requirement is reused for build validation; Node's
built-in runner tests presentation without npm packages.

All themes share DOM and behavior. Only semantic tokens vary. System resolves
platform Light/Dark; E-ink / Mono is an explicit high-readability monochrome theme.
Status words/icons/structure, visible focus, reduced motion, contrast and zoom
patterns establish a WCAG 2.2 AA baseline, not certification or a new review UI.
Zoom hotkeys grant only the main webview zoom permission, never filesystem/network
capabilities. Frontend remains presentation-only.

Extend D-021 with settings v2: database choices/enabled state are retained, with
`presentation.ui_locale` and `presentation.appearance` kept separate. Valid v1
migrates in memory with zh-HK/System defaults and no startup write. Only accepted
changes atomically write v2 under the same Rust lock, preserving failed-save,
concurrent-update and unsupported-document protections. New commands exchange only
typed presentation preferences and stable error codes, never the full document or
paths. UI language/theme never enter Runtime API v1.

This decision does not define localization defaults, Usage Context storage, mode
semantics, terminology precedence, review editing, dictionary management or
remember-choice UI. Those remain separately scoped later stages.

Canonical contributor, persistence and validation details:
[Desktop UI foundation](../DESKTOP_UI_FOUNDATION.md).
