# Rust Reference Engine

Phase 3A introduces the first Rust implementation of the shared deterministic localization runtime.

## Purpose

The Rust crate under `rust/` is a behavioral port of the existing Python reference engine. It is not yet the final Tauri/mobile runtime.

The immediate rule is **correctness before optimization**:

1. Python remains the behavioral oracle for the currently locked contract.
2. Rust reads the same SQLite v0.2 database.
3. Rust is tested against the same short evaluation cases and Phase 2E realistic synthetic corpus.
4. Output and review-needed state must match before performance or UI work is accepted.

## Current supported routes

- `zh-CN -> zh-HK`
- `zh-CN -> zh-TW`
- `zh-Hant -> zh-HK`
- `zh-Hant -> zh-TW`

The shared-core precedence follows D-009:

1. conservative entity matching;
2. protect resolved/review-required entity spans;
3. staged generic/script/regional rules;
4. longest applicable source match first;
5. highest priority within the selected source match;
6. preserve source and request review instead of guessing on ambiguity.

Current-version filtering uses the same SQLite v0.2 `source_versions.is_current` lifecycle as the Python reference implementation.

## Build and test

Build the shared fixture database first:

```bash
python scripts/build_demo_database.py
```

Then run:

```bash
cargo fmt --manifest-path rust/Cargo.toml -- --check
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path rust/Cargo.toml --all-targets
```

The integration tests read:

- `build/regional-demo.sqlite`
- `data/fixtures/evaluation_cases.json`
- `data/fixtures/realistic_corpus.json`

## CLI

Example:

```bash
cargo run --manifest-path rust/Cargo.toml -- \
  --db build/regional-demo.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "布拉德·皮特研究人工智能"
```

The CLI prints structured JSON.

## Deliberate Phase 3A limitations

Phase 3A does **not** yet claim full production-runtime parity. The first slice intentionally excludes:

- private `user_dictionary.sqlite` protected terms and user overrides;
- the Python engine's full provenance/alignment event payload shape;
- Tauri commands/UI bindings;
- mobile packaging;
- full upstream OpenCC runtime parity beyond the project's current tested reference subset;
- Rust-specific performance optimization.

These are later phases. They must not be implemented by weakening the deterministic/no-guess contract.
