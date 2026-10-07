# D-012 — User-local rules are separate and higher precedence

Date: 2026-10-07

User-local customization must not modify shared/open-data databases.

## Decision

- Store personal protected terms and fixed overrides in a separate `user_dictionary.sqlite`.
- Apply user-local rules before the shared deterministic engine.
- A matched user-local span is not passed through entity or shared terminology/script stages.

Precedence:
1. protected user term — preserve exact text;
2. user fixed override — use explicit replacement;
3. shared entity resolution;
4. shared regional terminology rules;
5. generic/script rules.

Within user rules:
- longest surface wins first;
- protected beats override for the same surface;
- more locale-specific scope beats a wildcard scope;
- then higher explicit priority wins;
- unresolved equal-rank conflicting overrides must be preserved and marked review-needed rather than guessed.

## Rationale

This keeps personal preferences private, avoids contaminating redistributed data packs, and makes the user's explicit instruction stronger than public knowledge-base defaults.
