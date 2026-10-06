# D-009 — Deterministic localization precedence and no-guess policy

Date: 2026-10-06

Core localization remains deterministic unless a later explicit decision supersedes this contract.

1. Match exact/longest known entity names or aliases in the declared source locale.
2. If one entity resolves and has one preferred target-regional name, use it and protect the span.
3. If entity resolution/target naming is ambiguous, preserve the source and mark review-needed.
4. Apply generic/script/regional rules in explicit stages; do not flatten all dictionaries.
5. Within a stage, longest source phrase wins before priority.
6. For the same source phrase, highest-priority candidate wins.
7. Equal-priority conflicting targets are not guessed; preserve and mark review-needed.
8. Applied/review-required decisions must remain explainable through provenance.

Initial routes:
- `zh-CN -> zh-HK`: entity, `zh-CN -> zh-Hant`, `zh-Hant -> zh-HK`.
- `zh-CN -> zh-TW`: entity, `zh-CN -> zh-Hant`, `zh-Hant -> zh-TW`.
- `zh-Hant -> zh-HK/TW`: entity then regional stage.
