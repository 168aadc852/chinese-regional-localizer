# Desktop Alpha Stage A — presentation foundation

Issue #60 is the narrow first implementation slice of #50. This static shell adds
UI language, appearance and accessibility patterns, not a redesigned Alpha workflow.
No localization engine, Runtime API v1, supported route, shared/private schema,
review-session, context hierarchy, mode or terminology-selection behavior changes.

## Edit translations without editing JavaScript

The human source of truth is [`ui-strings.csv`](../desktop/ui/i18n/ui-strings.csv).
It is UTF-8 CSV with `key,screen,note,zh-HK,zh-TW,zh-CN,en` columns. Open it in
GitHub, Excel (import as UTF-8) or LibreOffice. A UTF-8 BOM and normal LF/CRLF
line endings are accepted. Keep the header and stable semantic keys unchanged;
edit the appropriate locale column. Notes explain the location and meaning.
Quote cells containing commas, quotes or newlines using normal CSV escaping.

The four Chinese/English columns are authored independently: no runtime script
conversion and no silent fallback to another UI language. UI language is separate
from document Source/Target. The selector always displays each language's own name,
with no flags. Default UI locale is `zh-HK`; default appearance is `system`.

From the repository root:

```text
python scripts/check_ui_translations.py --generate
python scripts/check_ui_translations.py
node --test desktop/tests/presentation.test.cjs
python -m unittest discover -s tests -p test_ui_translations.py -v
```

Commit the CSV and generated files together. `i18n/locales/{locale}.json` and
`i18n/locales.js` are generated artifacts, not additional editing surfaces.
The classic-script bundle contains the same four dictionaries so the offline
Tauri asset loader needs no network fetch, module loader or npm framework.

Default validation is read-only, deterministic and fails on malformed CSV,
duplicate/invalid keys, missing values, placeholder mismatches, unknown literal
HTML/JS references, missing/stale/extra generated keys and extra locale files.
`--generate` validates input/references before writing generated resources.
Dynamic key maps register literal semantic keys with `key(...)`; runtime lookup
also fails on unknown keys or missing parameters rather than showing mixed locales.
Completion counts and missing keys are reported in plain text for contributors.

CI runs validation and focused presentation tests. Every Cargo desktop build also
runs the read-only Python gate, including direct release builds. Python 3.11 must
be available as `python`, or set `CRL_UI_PYTHON` to an installed executable. The
gate needs only the standard library. Node 22 is CI test tooling only, not an app
runtime/build requirement. Nothing installs automatically.

User document text, filenames, versions and existing technical provenance IDs are
data, not UI messages. The existing read-only v1 change list is retained; Stage A
does not reinterpret its review flag as the later Alpha Action State taxonomy.
Future review vocabulary/tokens are available but no alternative/remember UI is added.

## Themes and reusable accessibility baseline

`styles.css` owns semantic surface/text/border/action/focus/status/review tokens.
Components use tokens, never feature-specific colour literals. `system` follows
the platform/browser Light/Dark media preference live; it never selects mono.
`eink_mono` is explicit black/white/neutral styling with strong borders, no
gradients/filter/blur/transparency/shadow dependency and no decorative motion.
The structure and functionality are identical across themes.

Patterns for later stages:

- Native labelled selects, semantic buttons and named textareas preserve keyboard
  operation. Async display saves restore blurred focus without stealing focus
  if the user moved elsewhere. All representative controls have a visible 3px
  offset focus ring and at least 44px target height (above the 24px AA baseline).
- Status messages pair words with check/neutral/attention symbols and structure.
  Review attention uses a dashed border; errors use a double border. Never rely
  on colour alone, and never style Needs your decision as a blocking error.
- Action State, Candidate Rank and Your preference have separate token hooks.
  Future callers must supply meaningful text plus icon/shape; tokens do not
  classify content. No new modal/drawer is added in Stage A.
- Honour `prefers-reduced-motion`; mono also disables non-essential animation,
  transitions and decorative shadows. No separate persisted motion override.
- Rem sizing, wrapping headings/actions, shrinkable grid tracks and single-column
  breakpoints support text scaling and 200% zoom without a fixed desktop width.
  Tauri zoom hotkeys are enabled. The main-window capability grants only
  `core:webview:allow-set-webview-zoom` for Tauri's platform zoom polyfill;
  no filesystem, network or broad core capability is added.

Automated contrast checks cover both surfaces in every theme: text/status tokens
at least 4.5:1, border/focus/action boundaries at least 3:1, action text at least
4.5:1 including hover. These are baseline checks, not complete WCAG certification.
Full assistive-technology and workflow audits remain later Alpha QA work.

## Safe presentation persistence

Settings v2 preserves database fields and adds only:

```json
"presentation": { "ui_locale": "zh-HK", "appearance": "system" }
```

Rust validates enums and rejects unknown request fields. `presentation_settings`
returns only those two preferences; `save_presentation_settings` accepts only those
preferences. Full paths/settings documents stay in Rust. Presentation never enters
the localization Runtime request or persists source, target, Usage Context or mode.

Valid settings v1 migrate in memory with zh-HK/System defaults without a startup
write. Accepted changes write v2 under the same lock/atomic-replacement boundary
as database choices, preventing lost concurrent updates. Failed writes leave the
previous session/document unchanged; the UI restores accepted language/theme.
Corrupt/incomplete settings fall back with a warning; unsupported versions/fields
remain preserved and write-blocked, including unknown nested presentation fields.
Database restoration and private enabled-state protections remain unchanged.

New presentation errors are stable serialized codes: `preferences_unavailable`,
`settings_unsupported`, `settings_save_failed`. The UI supplies multilingual wording.
Legacy database/update/localization errors retain their backend contract; the shell
shows localized operation/status summaries instead of parsing or displaying raw
Chinese backend prose. More precise typed legacy diagnostics belong to the later
bridge stage, not a core/error-API redesign here.

An ordinary browser can preview presentation in memory only, visibly labelled
as not saved. It cannot localize, select databases or simulate the Rust core.

## Manual smoke checklist

Use the Windows debug executable built by Cargo with the local fixture database.
Check all four UI locales × Light/Dark/E-ink / Mono; inspect headings, controls,
status text, self-labels and focus. UI switching must retain document text/route
and must not rerun localization. Close/reopen to verify accepted preferences.
Use Tab/Shift+Tab, arrows and Enter for selectors/actions, and check 200% zoom
with wrapping/single-column layout. Verify System light/dark changes separately
(never mono). Test results for this implementation are recorded in the PR closeout;
mocked Node tests alone are not a manual smoke or screen-reader certification.

2026-10-10 Windows closeout: the real debug executable passed all 12 locale/theme
combinations, visible keyboard focus and keyboard-triggered localization at 200%
zoom, wrapping/single-column layout, and language/theme restore after restart.
System media changes and reduced-motion guards have automated coverage; Windows
OS preferences were not changed for this smoke. No screen-reader certification
or complete WCAG audit is claimed. Full automated results are in the PR closeout.
