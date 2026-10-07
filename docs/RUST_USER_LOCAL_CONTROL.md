# Rust User-Local Control

Phase 3B adds the private user-local control layer to the Rust reference runtime.

## Storage

Rust reads the existing separate `user_dictionary.sqlite` v0.1 database directly. The shared regional database and the user's private dictionary remain separate files.

Phase 3B is read-only from Rust: dictionary creation/editing remains outside this runtime slice. This avoids coupling local preference management to the shared data release database.

## Precedence

The Rust runtime follows the same user-local precedence as the Python reference implementation:

1. protected user term;
2. fixed user override;
3. shared entity localization;
4. regional terminology;
5. generic/script conversion.

Protected terms therefore prevent both entity localization and later terminology/script conversion for the matched surface.

## Matching and ranking

- User surfaces are matched longest-first.
- Only enabled rules are considered.
- Locale-scoped rules are filtered to the active source/target pair.
- More-specific source/target scope outranks a global rule.
- Priority breaks ties within the same specificity.
- Equal-ranked conflicting override replacements are not guessed: source text is preserved and `review_needed` is set.
- If a protected rule and override share the same surface, protected wins.

## Explanation output

User-local events include:

- event type (`user_protected` or `user_override`);
- reason;
- original/replacement text;
- source and target locale;
- `user_term_id` when a winning rule exists;
- optional note;
- `provenance: user_dictionary`;
- original-input and final-output character spans.

Shared-engine events emitted from text between user-controlled spans are offset back to global input/output character positions.

## CLI

The Rust CLI continues to work without a user database. To enable private user rules, add `--user-db`:

```bash
cargo run --manifest-path rust/Cargo.toml -- \
  --db build/regional-demo.sqlite \
  --user-db path/to/user_dictionary.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "OpenAI人工智能"
```

## Validation

Phase 3B adds Rust regressions corresponding to the existing Python user-localizer scenarios: protected/override precedence, locale isolation, longest surface, specificity, disabled rules, ambiguity/no-guess behavior and global spans.

The existing Phase 3A shared-core corpus remains part of the same Cargo test run.
