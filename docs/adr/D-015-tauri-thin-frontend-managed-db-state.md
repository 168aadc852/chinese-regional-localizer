# D-015 — Tauri frontend is thin and database paths stay in Rust state

Status: Accepted

Date: 2026-10-07

## Decision

The Tauri frontend is a thin adapter over Runtime API v1. It may submit localization requests and render responses, but it must not duplicate localization precedence, query SQLite directly, or supply arbitrary shared/user database paths to the localization command.

Shared and optional private user-dictionary paths are owned by Rust managed state. Phase 3D development builds may initialize those paths from trusted process environment variables at application startup.

## Rationale

The deterministic localization contract already lives in the Rust Runtime API. Reimplementing any part of it in JavaScript would create two behavior paths and make parity, desktop/mobile reuse and later maintenance harder.

Keeping database locations behind the Rust command boundary also reduces the frontend's authority and leaves future database selection, validation, updates and migration in one native layer.

## Consequences

- Frontend JavaScript sends only Runtime API request data.
- Tauri commands call Runtime API v1 rather than lower-level SQL or replacement logic.
- Database settings/updater work must be implemented on the Rust side.
- A future settings UI may request a Rust-side path-selection operation, but it should not pass arbitrary paths directly to localization commands.
- The same runtime boundary can later be reused by mobile or other native shells.
