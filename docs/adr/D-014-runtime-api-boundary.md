# D-014 — Runtime API is the application boundary

Date: 2026-10-07

The versioned Rust `Runtime` API is the application-facing boundary for future Tauri commands and other product surfaces.

Application/UI code must not reimplement localization precedence, entity resolution, term-rule selection, ambiguity handling or user-dictionary precedence. It should submit a runtime request and render the runtime response.

Runtime API v1 must preserve:

- deterministic/no-guess behavior;
- shared-only and optional private user-dictionary execution;
- route/output/review state;
- structured change explanations;
- UI-oriented original-input/final-output character spans;
- entity and term-rule provenance where available.

The lower-level Rust shared engine and user-local layer remain internal implementation components. Their internals may be optimized later as long as the tested v1 behavior remains compatible.

Hosted CI benchmark measurements are regression signals only and do not define product performance guarantees.
