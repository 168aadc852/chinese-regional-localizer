# D-019 — Authenticated network update boundary stays in Rust

Status: Accepted

## Decision

Network discovery/download for shared-data packages is owned by the Rust runtime, not frontend JavaScript.

A production update origin must use HTTPS. Redirects are disabled. Catalog and package URLs are derived from one configured origin, not accepted as arbitrary frontend-supplied URLs.

Before any downloaded database can become active, the runtime must enforce the existing trust chain:

1. pinned Ed25519 verification of release catalog;
2. catalog expiry and monotonic sequence checks;
3. package-manifest signature verification;
4. exact catalog-to-manifest SHA-256 and identity binding;
5. signed database size/SHA-256 checks;
6. existing SQLite/package validation;
7. immutable PackageStore installation/activation.

The highest trusted catalog sequence is persisted locally so restart cannot reset rollback protection. A failed discovery or install must not alter the active package.

## Rationale

HTTPS protects transport but is not sufficient release authenticity. Keeping URL construction, download limits, signature verification and package activation in Rust prevents the frontend from becoming a generic network/filesystem bridge and preserves the offline-first trust boundary already established by D-015 through D-018.

## Consequences

- localization remains fully offline;
- update networking can be added without weakening the thin-frontend model;
- CI can test the trust flow using a fake transport without Internet access;
- future UI commands must remain narrow (for example check/install known updates) rather than accept arbitrary URLs;
- remote key rotation, background scheduling and application-binary updates require separate decisions.
