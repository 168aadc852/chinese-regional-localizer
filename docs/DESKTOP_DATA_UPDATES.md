# Desktop Authenticated Data Updates

Phase 3I exposes the Phase 3H authenticated shared-data update flow through narrow Tauri commands and a small desktop UI.

## Security boundary

The frontend cannot provide an update URL, trusted key, package-store path or package ID. Those values are Rust-side startup configuration only:

- `CRL_UPDATE_BASE_URL`
- `CRL_TRUSTED_KEYS_FILE`
- `CRL_UPDATE_STORE`
- `CRL_UPDATE_PACKAGE_ID`

All four values must be present for update controls to be enabled. If they are absent, the application remains fully usable offline and reports that the update service is not configured.

The trusted-key JSON may be either an array of `TrustedKey` records or an object containing a `keys` array. Full filesystem paths are not returned to frontend JavaScript.

## User flow

1. The UI calls `update_status` to display local package/update state.
2. The user explicitly presses **Check for updates**.
3. Rust creates the Phase 3H HTTPS client and downloads the signed release catalog.
4. Rust verifies catalog signature, expiry and sequence rollback protection.
5. The desktop layer selects the last runtime-compatible entry for the configured package ID in the authenticated catalog, excluding the currently active version.
6. The UI receives only structured status: configured/current version/offered version/availability/message.
7. The user explicitly presses **Install update**.
8. Rust reuses the authenticated catalog and Phase 3H install path: signed manifest binding, bounded database download, size/hash validation, SQLite/package validation and immutable `PackageStore` activation.
9. Only after successful validation/activation does the desktop runtime switch its shared database path to the newly active package.

A failed check or install does not replace the current shared database.

## Catalog ordering

Phase 3I does not interpret arbitrary version strings as semantic versions. For a configured package ID, the signed release catalog order is authoritative and the final compatible entry that is not the currently active version is treated as the offered package. This keeps version policy under the signed publisher catalog rather than guessing from version text.

## Not included

- background/scheduled checks;
- retry/resume for interrupted downloads;
- production CDN/hosting layout;
- application binary updates;
- remote trust-root rotation/delegation;
- private signing-key operations.
