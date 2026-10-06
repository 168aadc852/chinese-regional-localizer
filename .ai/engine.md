# Localization Engine Tasks

Start with:
- `src/localizer_engine.py` or the production-engine file being changed;
- the nearest engine tests;
- `docs/adr/D-009-localization-precedence.md`.

Load on demand:
- `docs/LOCALIZER_ENGINE.md` for detailed behavior;
- `docs/OPENCC_IMPORTER.md` only when OpenCC semantics are involved;
- `docs/DATABASE_SCHEMA.md` only when queries/model assumptions change.

Core contract:
- entity resolution before generic rules;
- longest match before priority;
- no guessing on ambiguity/conflict;
- protect resolved/review-required spans;
- preserve staged CN→Hant→HK/TW behavior;
- return explainable provenance.

Do not preload licensing/source-review docs unless new external data is introduced.
