# Phase 3E security boundary

The desktop settings UI may request narrowly defined configuration actions, but it must not receive direct filesystem traversal, arbitrary path execution, or SQLite access. Rust owns path validation, schema compatibility checks, and runtime configuration state.
