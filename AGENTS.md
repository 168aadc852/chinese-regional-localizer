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
- README / public product copy: `.ai/readme-style.md`
- Release / packaging: `.ai/release.md`

Read `README.md`, project overview, product requirements, history or unrelated technical docs only when the task actually requires them.

## README communication policy

`README.md` is product-facing and Chinese-user-first.

When updating README:
- Traditional Chinese product/user content comes first; English and developer details are secondary.
- Keep the opening concise and product-oriented.
- Explain HanContext as deterministic, offline-first, controllable, explainable and context-aware.
- Explain non-LLM core advantages without framing HanContext as anti-AI or "better than LLMs".
- Keep unfinished Alpha features clearly marked as planned or in progress.
- Preserve accurate supported-route claims.
- Use `Chinese Mainland` in English regional wording.
- Load `.ai/readme-style.md` only when working on README or public product copy.

## Global rules

- Do not guess licensing facts or ingest unapproved data.
- Do not silently change established architecture or localization behavior.
- Preserve offline-first and distinct `zh-CN` / `zh-HK` / `zh-TW` goals.
- Run the smallest relevant tests first; run the full relevant suite before declaring implementation complete.
- Write material decisions, source-review results and meaningful progress back to the repository.
- At every major phase closeout, and whenever user-facing capabilities or the public project status materially change, review `README.md` and update it in the same PR when it is no longer accurate.
- If context is missing, load the next relevant file on demand instead of preloading the documentation tree.
