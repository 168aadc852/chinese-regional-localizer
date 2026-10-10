# README and Public Copy Style

## Purpose

The main README is primarily for Chinese-speaking users evaluating HanContext. It is not primarily an engineering status log, architecture document, phase-history record or implementation dump. Those details belong in project-state and technical documentation.

Use this file only when editing `README.md` or other public product copy.

## Content priority

Preferred order:

1. Product name and positioning
2. What problem HanContext solves
3. Why the deterministic / non-LLM core matters
4. Current capabilities and current progress
5. Supported localization routes
6. How HanContext works at a user level
7. Data-source transparency and licensing
8. Project status / roadmap links
9. Developer information
10. Short English summary

Keep implementation internals out of the opening sections.

## Language priority

Primary:
- Traditional Chinese
- Hong Kong-oriented written Chinese

Secondary:
- English for international developers and contributors

Rules:
- Avoid Cantonese colloquial written forms in README public copy.
- In English regional wording use `Chinese Mainland`, never `Mainland China`.
- Use `中國內地` as the product-level Chinese regional label unless a locale-specific UI contract explicitly requires otherwise.
- Do not use country or city flags for language, locale or region presentation.

## Product positioning

Preferred messages include:

- 懂情境的中文地區化
- 同一種中文，不只換字，更要換對語境。
- 盡量保留原句，只處理真正需要地區化的內容。
- 寧願保留原文，也不要自行猜測。
- Same rules. Same result. Every time.
- Your terms. Your context. Your Chinese.

Describe HanContext as:
- deterministic
- offline-first
- controllable
- explainable
- context-aware

## Non-LLM positioning

The core does not require an LLM. Explain the benefits in terms of consistency, reproducibility, auditability, privacy, offline operation, stable terminology and independence from token/API/model availability.

Do not frame HanContext as anti-AI, as categorically better than LLMs, or as a replacement for every AI-assisted workflow. AI may be described as an optional assistance layer while the deterministic core remains independently usable.

Prefer wording such as:
- `no LLM required`
- `without requiring an LLM`

Avoid superiority claims such as:
- `better than LLMs`
- `more intelligent than AI`

## Progress and accuracy rules

Always distinguish these states:

1. Implemented core capability
2. Implemented Desktop UI capability
3. Formally approved design / planned Alpha capability
4. Future roadmap capability

Rules:
- Do not claim a feature is complete before it is merged.
- Do not describe approved UX design as already implemented Desktop functionality.
- Do not imply unsupported localization routes.
- Do not let implementation detail dominate the opening section.
- Keep future work explicitly labelled as planned, in progress or proposed.

For data sources, preserve this distinction:

`reviewed ≠ ingest allowed ≠ exact resource pinned ≠ importer ready ≠ ingested ≠ validated ≠ included in an official pack`

Never describe reviewed sources as included production data unless they are actually packaged and validated for release.

## Supported routes

Only claim the currently supported forward routes:

- `zh-CN → zh-HK`
- `zh-CN → zh-TW`
- `zh-Hant → zh-HK`
- `zh-Hant → zh-TW`

Do not imply arbitrary locale pairs or target `zh-CN` support.

`zh-Hant` means generic Traditional Chinese input, not a specific Hong Kong or Taiwan source locale.

## Product and UX terminology

Prefer plain-language product terminology. Current approved examples include:

- Usage Context / 使用情境
- My Terms
- My Dictionaries
- Standard localization / 標準地區化
- Conservative localization / 保守地區化
- Script only / 只轉字形
- Recommended
- Also valid
- Needs your decision
- Your preference
- Official Dictionary
- Data Sources

Do not expose raw rule IDs, numeric priorities, confidence scores, database schema details or internal packaging terminology in the main user-facing README unless a technical section specifically requires them.

## Source transparency

HanContext should not hide dictionary/data sources from users, but the README should present source information progressively and accurately.

Prefer:
- what the Official Dictionary is
- that users can inspect included sources
- that third-party licensing is reviewed separately
- that reviewed sources are not automatically included

Avoid implying that users must manually download upstream dictionaries for normal Desktop use.

## README maintenance

Review `README.md` whenever a major product-facing milestone is merged or public status materially changes.

When updating README:

1. Check whether the progress/status section is stale.
2. Update only facts already implemented or formally approved.
3. Keep future work explicitly labelled as planned or in progress.
4. Avoid rewriting unrelated sections without a product reason.
5. Preserve supported-route and data-source accuracy.
6. Prefer links to `PROJECT_STATE.md`, product requirements, roadmap and technical documentation instead of copying detailed implementation history into README.

Do not accumulate technical history in the README opening.
