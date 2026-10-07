# Release Signing and Trusted Metadata

Phase 3G adds publisher-authenticity checks before any network updater is allowed.

Phase 3F SHA-256 checks answer: **did these bytes change?** Phase 3G Ed25519 signatures answer: **were these bytes signed by a pinned trusted release key?** Both layers remain required.

## Trust model

The application trusts only public keys pinned by the application/release configuration. Each trusted key contains:

- `algorithm: ed25519`;
- the base64-encoded 32-byte public key;
- a key ID derived as `ed25519-sha256:<SHA-256(public-key-bytes)>`.

A key ID is only a selector/fingerprint. Trust comes from the pinned public-key bytes. Multiple pinned keys are supported so a future app release can overlap old/new verification keys during planned rotation.

Remote metadata cannot add a new trusted root key in Phase 3G.

## Detached signature envelope v1

Signatures are stored separately from the signed payload:

```json
{
  "signature_version": 1,
  "algorithm": "ed25519",
  "key_id": "ed25519-sha256:...",
  "signature_base64": "..."
}
```

Verification uses `ed25519-dalek` strict Ed25519 verification.

The exact raw bytes are signed. Reformatting JSON after signing invalidates the signature.

## Domain separation

Package manifests and release catalogs use different signed-message prefixes:

- `CRL-PACKAGE-MANIFEST-V1\0`
- `CRL-RELEASE-CATALOG-V1\0`

This prevents a valid signature for one metadata type from being reused as a signature for the other type.

## Release catalog v1

`release-catalog.json` contains:

- `catalog_version`;
- monotonic `sequence`;
- `generated_at_unix`;
- signed `expires_at_unix`;
- package entries containing package/version/pack/runtime compatibility and SHA-256 of the exact `package.json` bytes.

The Rust verifier rejects:

- bad/unknown signatures;
- unknown key IDs;
- expired catalogs;
- a sequence lower than the highest previously trusted sequence;
- malformed/duplicate package entries;
- package manifests whose exact bytes or identity no longer match the signed catalog entry.

The sequence check is rollback protection. Signed expiry limits how long a stale catalog can remain acceptable.

## Offline release workflow

1. Build and validate a Phase 3F package.
2. Build `release-catalog.json` with `scripts/build_release_catalog.py`.
3. On a controlled signing machine, sign the exact `package.json` and `release-catalog.json` bytes with the Rust `sign_metadata` utility.
4. Publish payloads plus detached signature envelopes.
5. A future downloader must verify trusted signed metadata **before** passing a downloaded package into the Phase 3F install/activate path.

Example signing command:

```text
cargo run --locked --manifest-path rust/Cargo.toml --bin sign_metadata -- \
  --kind catalog \
  --input release-catalog.json \
  --secret-key-file /secure/external/location/release-seed.b64 \
  --output release-catalog.sig.json
```

The secret-key file contains base64 for exactly 32 Ed25519 seed bytes. A production private key must never be committed to this repository, copied into application resources, printed in logs, or distributed with release artifacts.

The CLI can optionally export the corresponding public-key record for pinning with `--public-key-output`.

## Current boundary

Phase 3G is still offline infrastructure. It does **not** add:

- HTTP update discovery/download;
- remote root-key rotation/delegation;
- HSM/cloud key-vault integration;
- application/installer code signing;
- mobile delivery.

A future network updater must preserve both Phase 3G authenticity checks and Phase 3F package/SQLite validation and rollback. It must not bypass either layer.
