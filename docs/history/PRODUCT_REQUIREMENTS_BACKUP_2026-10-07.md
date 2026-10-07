# Product Requirements Backup — 2026-10-07

This file is a frozen documentation backup of `docs/PRODUCT_REQUIREMENTS.md` as of 2026-10-07.

Purpose:
- preserve the product-requirement baseline separately from implementation history;
- provide a stable human-readable recovery/reference copy;
- avoid mixing requirement backup with code or runtime behavior.

This backup does **not** supersede `docs/PRODUCT_REQUIREMENTS.md`. The live requirements file remains the canonical editable source.

---

# Initial Product Requirements

Status: draft

## Core localization targets

- Auto-detect source variety where practical.
- Convert to `zh-CN`.
- Convert to `zh-HK`.
- Convert to `zh-TW`.

## Conversion levels

Proposed user-facing levels:

1. **Script only** — character/script conversion.
2. **Standard localization** — script plus common regional terminology.
3. **Full localization** — includes proper names and entity-aware localization.

## Explainability

For significant changes, the application should be able to show:

- original text;
- localized result;
- change type;
- data source;
- source record/revision when available;
- confidence or ambiguity status.

## Review workflow

Users should be able to:

- accept/reject uncertain changes;
- protect a word from conversion;
- define a preferred local term;
- add a user dictionary override;
- review only high-impact or uncertain changes.

## Multi-region comparison

A lookup/comparison mode should show CN / HK / TW forms for a term or entity where known.

## Offline operation

Normal text localization should work without Internet access once required data packs are installed.

Internet access may be used for explicit database/application updates.

## Future file support

Candidate order:

1. Plain text / clipboard
2. Markdown / TXT / subtitles
3. CSV
4. Office documents
5. EPUB and other structured formats

## Platform direction

Long-term target:

- Windows
- macOS
- Android
- iOS / iPadOS

A shared core should be preferred over independent per-platform conversion logic.
