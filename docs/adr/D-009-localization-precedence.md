# D-009 — Deterministic localization precedence and no-guess policy

Date: 2026-10-06  
Hardened: 2026-10-07

Core localization remains deterministic unless a later explicit decision supersedes this contract.

1. Entity matching uses current evidence only and a conservative surface policy; very short/common names are not automatically treated as entities merely because they exist in the database.
2. Within eligible entity surfaces, longest match is considered first.
3. If one entity resolves and has one current preferred target-regional name, use it and protect the span.
4. If entity resolution/target naming is ambiguous, preserve the source and mark review-needed.
5. Apply generic/script/regional rules in explicit stages; do not flatten all dictionaries.
6. Within a stage, longest applicable source phrase wins before priority.
7. Rule `constraints` are enforced before ranking; malformed constraint JSON is not treated as unrestricted.
8. For the same source phrase, highest-priority candidate wins.
9. Equal-priority conflicting targets are not guessed; preserve and mark review-needed.
10. Applied/review-required decisions retain provenance plus original-to-final span alignment.

Initial routes remain:
- `zh-CN -> zh-HK`: entity, `zh-CN -> zh-Hant`, `zh-Hant -> zh-HK`.
- `zh-CN -> zh-TW`: entity, `zh-CN -> zh-Hant`, `zh-Hant -> zh-TW`.
- `zh-Hant -> zh-HK/TW`: entity then regional stage.
