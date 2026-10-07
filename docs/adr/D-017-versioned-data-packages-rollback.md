# D-017 — Versioned immutable data packages with staged activation and rollback

Status: Accepted

## Context

The desktop runtime is ready to consume real shared databases, but replacing an active SQLite file in place would make interrupted or invalid updates dangerous. Data packaging must also preserve the project's existing licence/source governance.

## Decision

Redistributable shared databases are delivered as versioned package directories with a `package.json` manifest and one referenced SQLite artifact.

- Package creation re-validates machine-readable source policy and licence-pack consistency.
- Runtime installation independently validates manifest compatibility, safe paths, byte size, SHA-256, SQLite integrity, required tables and pack metadata.
- New versions are copied to staging and validated before activation.
- Installed versions are immutable by convention and live under `<store>/packages/<package_id>/<version>/`.
- Active state is stored separately as `current` and `previous` references.
- The active database is never overwritten in place.
- Failed installation does not alter the active state.
- Rollback validates the previous package before swapping active pointers.

SHA-256 is an integrity mechanism, not publisher authentication. Release signing is a separate future decision.

## Consequences

A broken or incomplete package cannot silently replace a working database. Update discovery/download can be added later without changing the fundamental install/activate/rollback safety contract.