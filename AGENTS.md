# AI Instructions

This repository is the project source of truth.

## Context budget

Do **not** preload project documentation. Start from the current task or GitHub Issue, then read only the files needed to complete it safely.

Prefer: specific file/section > task guide > index > broad project documentation.

## Task router

- Current status only: `PROJECT_STATE.md`
- Code changes: `.ai/coding.md`
- Data-source / licence review: `.ai/data-review.md`
- Importer work: `.ai/importer.md`
- SQLite / schema work: `.ai/database.md`
- Localization behavior: `.ai/engine.md`
- Tests / CI: `.ai/testing.md`
- Documentation changes: `.ai/docs.md`
- Release / packaging: `.ai/release.md`

Read `README.md`, project overview, product requirements, history or unrelated technical docs only when the task actually requires them.

## Global rules

- Do not guess licensing facts or ingest unapproved data.
- Do not silently change established architecture or localization behavior.
- Preserve offline-first and distinct `zh-CN` / `zh-HK` / `zh-TW` goals.
- Run the smallest relevant tests first; run the full relevant suite before declaring implementation complete.
- Write material decisions, source-review results and meaningful progress back to the repository.
- If context is missing, load the next relevant file on demand instead of preloading the documentation tree.
