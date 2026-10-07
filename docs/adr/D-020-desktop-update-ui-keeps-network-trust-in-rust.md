# D-020 — Desktop update UI keeps network trust in Rust

Status: Accepted

## Decision

The desktop frontend may trigger only narrow update actions: read update status, check the configured authenticated catalog, and install the currently offered package.

Frontend JavaScript must not receive or submit arbitrary update URLs, trusted keys, package-store paths, package IDs or generic HTTP requests. Update origin and trust-root configuration remain Rust-side startup configuration.

A package becomes the active shared database only after the existing authenticated update pipeline has verified catalog/signature/sequence, package-manifest binding, bounded download, signed size/SHA-256, SQLite/package validity and PackageStore activation.

## Rationale

The Tauri webview is presentation code, not a trust boundary. Keeping network destination selection, trust roots and activation in Rust prevents a frontend compromise or UI bug from redirecting the updater to an attacker-controlled source or bypassing package validation.

## Consequences

- The app can remain fully offline when update configuration is absent.
- Product releases must provide update origin, trusted-key material, package-store location and package identity through trusted application configuration.
- The frontend receives only structured status and cannot expose full local paths.
- Background checks, retry/resume and trust-root rotation require later explicit decisions.
