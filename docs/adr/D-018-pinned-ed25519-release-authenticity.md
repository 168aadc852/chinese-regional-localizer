# D-018 — Pinned Ed25519 release authenticity before network updates

Status: Accepted

## Context

Phase 3F validates package integrity and supports rollback, but SHA-256 alone cannot prove who published a package. A future network updater must not trust metadata merely because its hashes are internally consistent.

## Decision

Release metadata authenticity uses Ed25519 signatures verified against public keys pinned by the application/release configuration.

- Cryptography comes from a maintained library; project code does not implement Ed25519 primitives.
- Detached signature envelope v1 identifies `ed25519` plus a key fingerprint ID.
- Package manifests and release catalogs are signed over exact raw bytes with separate domain prefixes.
- Trusted key IDs are derived from pinned public-key bytes; remote metadata cannot create trust in a new root key.
- Multiple pinned keys may coexist for planned rotation.
- Release catalog v1 carries a monotonic sequence and signed expiry.
- Catalog sequences lower than the highest previously trusted value are rejected as rollback.
- Expired catalogs are rejected to limit stale/freeze metadata.
- Catalog package entries bind the exact package-manifest SHA-256 and identity fields.
- Production private signing keys never live in the source repository or application bundle.

## Consequences

Any future download layer must authenticate the catalog/package metadata first and then pass the package through the existing Phase 3F checksum/SQLite/staged-install validation. Neither layer replaces the other.

Remote trust-root rotation, threshold signatures/HSMs and installer signing remain separate future decisions.