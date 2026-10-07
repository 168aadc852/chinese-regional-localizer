# Authenticated Network Updates

Phase 3H adds the first network transport layer for shared-data updates. The product remains offline-first: localization itself never needs the Internet, and update networking is a separate explicit path.

## Trust flow

The runtime does not download a database and trust it merely because it came from HTTPS.

1. Fetch `release-catalog.json` and its detached signature from the configured HTTPS origin.
2. Verify the catalog against the pinned Ed25519 trusted-key set.
3. Reject expired catalogs and catalog-sequence rollback.
4. Persist the highest trusted catalog sequence locally.
5. Select only a package identity/version present in that authenticated catalog.
6. Fetch `package.json` plus its detached signature.
7. Verify the package-manifest signature and its exact SHA-256/identity binding to the signed catalog entry.
8. Only then fetch the database named by the authenticated manifest.
9. Check the signed size/SHA-256 and run the existing SQLite/package validation.
10. Install through the existing immutable `PackageStore`, which keeps current/previous activation state and rollback support.

Any failure before step 10 leaves the active package unchanged.

## Network boundary

Production configuration requires HTTPS. Redirects are disabled by the Rust HTTP client. Derived package URLs must remain on the configured host and port. URL credentials, query strings and fragments are rejected on the configured base URL.

The runtime applies bounded response sizes to catalog, signature, manifest and database downloads. It also checks `Content-Length` when provided and independently bounds streamed bytes, so a missing or false size header cannot bypass the limit.

Frontend JavaScript does not receive a generic HTTP client or arbitrary URL parameter. Phase 3H is a Rust-core API intended to be wired to a narrow application command later.

## Local rollback state

`update-state.json` lives beside the local package store and records `highest_trusted_catalog_sequence`. It is written using a temporary file followed by rename. Once a newer authenticated sequence has been accepted, an older signed catalog is rejected after restart as well.

This state is separate from package activation state. Discovering a valid newer catalog does not by itself replace the active database.

## Failure isolation

Downloaded package files are written into a temporary network staging directory. The staging directory is validated before `PackageStore::install` is called and is removed afterward. Invalid signatures, catalog binding, size/hash, SQLite integrity or package metadata therefore cannot replace the active database.

## Testing

CI uses an in-memory fake transport rather than the public Internet. Regression tests cover:

- HTTPS/base-origin validation;
- persisted catalog-sequence rollback protection;
- authenticated package installation;
- corrupted database isolation from the active package;
- invalid catalog signatures not advancing trusted sequence.

## Not implemented in Phase 3H

- background or scheduled update checks;
- a user-facing update button/status UI;
- application-binary/installer updates;
- remote trust-root rotation/delegation;
- production release hosting/CDN configuration;
- automatic retry/resume for interrupted large downloads.
