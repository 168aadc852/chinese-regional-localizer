# README and Public Copy Style

## Purpose

The main README is primarily for Chinese users evaluating HanContext. It is not primarily an engineering status log, architecture document or phase-history record. Those details belong in project-state and technical documentation.

## Content priority

Preferred order:

1. Product name and positioning
2. What problem HanContext solves
3. Why the deterministic / non-LLM core matters
4. Supported localization routes
5. Current Alpha user-facing status
6. Licensing / data note
7. Developer information
8. Short English summary

## Language priority

Primary:
- Traditional Chinese
- Hong Kong-oriented written Chinese

Secondary:
- English for international developers and contributors

Avoid Cantonese colloquial written forms in README public copy.

## Product positioning

Preferred messages include:

- 懂情境的中文地區化
- 同一種中文，不只換字，更要換對語境。
- 盡量保留原句，只處理真正需要地區化的內容。
- 寧願保留原文，也不要自行猜測。
- Same rules. Same result. Every time.

Describe HanContext as deterministic, offline-first, controllable, explainable and context-aware.

## Non-LLM positioning

The core does not require an LLM. Explain the benefits in terms of consistency, reproducibility, auditability, privacy, offline operation, stable terminology and independence from token/API/model availability.

Do not frame HanContext as anti-AI or as categorically better than LLMs. AI may be described as an optional assistance layer while the deterministic core remains independently usable.

## Accuracy rules

- Do not claim a feature is complete before it is merged.
- Clearly separate implemented core capability, desktop UI exposure and planned Alpha capability.
- Do not imply unsupported localization routes.
- In English regional copy use `Chinese Mainland`, not `Mainland China`.
- Do not let implementation detail dominate the opening section.

## README maintenance

Review README whenever a major product-facing milestone is merged or public status materially changes.

Do not accumulate technical history in the README opening. Prefer links to `PROJECT_STATE.md`, product requirements, roadmap and technical documentation instead of copying detailed implementation history into README.
