# Tauri Desktop Development (Phase 3D)

This is the first development-only desktop shell for Chinese Regional Localizer. It uses Tauri 2 with a static HTML/CSS/JavaScript frontend and the existing Rust Runtime API v1.

Issue #60 adds [four-language UI, themes and accessibility foundation](DESKTOP_UI_FOUNDATION.md)
without changing the localization core. It remains a development shell, not the complete Alpha workflow.

## Windows 11 beginner setup

1. Install Microsoft Visual Studio Build Tools 2022 and select **Desktop development with C++**.
2. Install Rust from `https://rustup.rs` and accept the default options.
3. Install the Microsoft WebView2 Runtime if Windows does not already have it. Current Windows 11 installations normally include it.
4. Open PowerShell in the repository folder.
   Python 3.11 must be available as `python`; every Cargo desktop build now runs the
   read-only translation validation gate. If needed, set `CRL_UI_PYTHON` to the full
   path of your installed Python executable. No Python packages are needed for this gate.
5. Build the development database first:

```powershell
python scripts/build_demo_database.py
```

6. Start the Tauri app from the repository root:

```powershell
cargo run --manifest-path desktop/src-tauri/Cargo.toml
```

The development shell defaults to `build/regional-demo.sqlite` in this repository.

The checked-in `desktop/src-tauri/icons/icon.ico` embeds the existing 128×128
`icon.png` unchanged. Tauri requires this Windows resource icon during builds,
even when installer bundling is disabled; keep both assets in the repository.

## Optional private user dictionary

The frontend is not allowed to choose arbitrary database paths. For development, configure the private dictionary before starting the app:

```powershell
$env:CRL_USER_DB = "C:\path\to\user_dictionary.sqlite"
cargo run --manifest-path desktop/src-tauri/Cargo.toml
```

To use another shared database:

```powershell
$env:CRL_SHARED_DB = "C:\path\to\regional.sqlite"
cargo run --manifest-path desktop/src-tauri/Cargo.toml
```

These paths are read by Rust at startup and stored in Tauri managed state. JavaScript only sends `text`, `source_locale`, `target_locale`, API version and optional localization context.

## Architecture

`desktop/ui/` contains only static frontend files. `desktop/src-tauri/` contains the native shell and narrow localization/database/update/presentation commands. `localize_text` calls `Runtime::localize` from Runtime API v1; the frontend does not reproduce localization rules.

The static frontend uses Tauri's global JavaScript API (`window.__TAURI__.core.invoke`) so this first shell does not require Node.js, npm, Vite, React or another frontend build chain.

Node.js 22 is used in CI for the built-in `node --test desktop/tests/presentation.test.cjs`
presentation tests only; there are no npm packages or frontend framework dependencies.
Edit `desktop/ui/i18n/ui-strings.csv`, then run `python scripts/check_ui_translations.py --generate`
and `python scripts/check_ui_translations.py` before building.

## Current limitations

This phase is a development shell, not a production installer. There is no installer signing, automatic database updater, packaged production database, mobile build or graphical custom-dictionary editor yet.
