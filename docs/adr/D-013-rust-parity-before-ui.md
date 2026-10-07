# D-013 — Rust parity before UI and optimization

Date: 2026-10-07

The Python reference engine remains the behavioral oracle while the production runtime is ported to Rust.

Before Rust becomes the application runtime or is connected to Tauri UI, it must demonstrate parity against the same SQLite fixture database and the same locked evaluation corpus used by Python.

Required order:

1. match supported routes and deterministic precedence;
2. match exact expected text output;
3. match review-needed/no-guess behavior;
4. preserve current-version filtering and conservative entity matching;
5. only then optimize Rust-specific performance;
6. only then expose the stable Rust runtime through Tauri commands/UI.

A faster implementation that changes expected localization behavior is a regression, not an optimization.

Phase 3A initially covers the shared localization core. User-local dictionary parity and the full explanation/alignment payload may follow as explicit later work, but they must preserve the already locked shared-core behavior.
