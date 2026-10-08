# HanContext Product Requirements

Updated: 2026-10-08. Status: consolidated direction; Alpha features remain planned.

**HanContext — Chinese, localized with context. / 懂情境的中文地區化**.
A controllable, explainable, deterministic, context-aware regional localization tool
for the Chinese Mainland, Hong Kong and Taiwan, going beyond character conversion.
Use **Chinese Mainland** in future UI, documentation, issues and product copy.
The repository name and application identifier remain unchanged.

First finish Phase 3J / Issue #38 ([PR #45](https://github.com/168aadc852/chinese-regional-localizer/pull/45),
pending review/CI/merge) as scoped; no Alpha feature belongs in that PR.
Next review [the Alpha MVP specification](HANCONTEXT_ALPHA_MVP.md) and its small issues.
Planning/issue creation is not blanket approval to implement features.
Implemented capabilities/routes are recorded in `../README.md` and `../PROJECT_STATE.md`;
these requirements describe intended behavior.

## Core localization targets

- Eventually auto-detect source variety where practical; initial Alpha uses explicit supported routes.
- Convert to `zh-CN`.
- Convert to `zh-HK`.
- Convert to `zh-TW`.

## Conversion levels

Proposed user-facing levels:

1. **Script only** — character/script conversion.
2. **Standard localization** — script plus common regional terminology.
3. **Conservative localization** — prefer leaving ambiguous wording unchanged.

Standard is the proposed default. Do not suggest “Full” is always more correct.
Exact mode semantics and entity/protected-term interactions need separate approval and tests.

## Explainability

For significant changes, the application should be able to show:

- original text;
- localized result;
- change type;
- why the wording changed, in plain language;
- Recommended / Also valid / Needs your decision where appropriate.

Technical provenance, datasets/records and rule/data versions belong in a deeper
optional layer. Main UI must not lead with raw source/dictionary names, database
priority, rule IDs, confidence scores, provenance schemas or raw tables.

## Review workflow

Users should be able to:

- accept/reject uncertain changes;
- protect a word from conversion;
- define a preferred local term;
- add a user dictionary override;
- review only high-impact or uncertain changes.

The intended workflow is input → source region → target region → usage context
→ localization behaviour → localize → review → inspect important changes
→ choose valid alternatives → optionally remember a preference.

Alternatives should update the selected occurrence without rerunning the document.
Offer Use this time only / Remember for this context / Remember for all contexts.
Define stable span/undo handling in a separate alternatives issue.

## Usage contexts and My Terms

Possible built-ins: General; Technology / Software; Banking / Finance; Business /
Marketing; Legal; Education; Government / Public Administration. Profiles do not
promise terminology coverage or authorize ingestion of unapproved data.
Custom profiles such as Hi-Fi Audio, ESG and University Administration should inherit
a general context rather than duplicate every rule; approve the inheritance model first.

The private dictionary should evolve toward **My Terms**. Preferences may depend on
source term, preferred term, target locale and context, not one permanent global
replacement. Present them as “Use X instead of Y when writing Hong Kong Chinese
for Technology / Software.” Keep priority numbers internal.

Proposed future precedence:

1. Exact-context user preference.
2. All-context user preference.
3. Governed current-context terminology.
4. General target-region terminology.
5. Script / character conversion.

A separately approved tested design must reconcile this direction and inheritance
with existing protected-term, entity and longest-match rules; do not silently replace
the current contract. Never choose equal-level valid candidates arbitrarily; preserve
original text when no safe decision exists.

The deterministic core needs no LLM. Identical input/settings/context/data-version
combinations produce reproducible output. Future AI assistance is optional, especially
for unresolved ambiguity, and must not become a core requirement.

## Multi-region comparison

A lookup/comparison mode should show CN / HK / TW forms for a term or entity where known.

## Offline operation

Normal text localization should work without Internet access once required data packs are installed.

Internet access may be used for explicit database/application updates.

## Future file support

Candidate order:

1. Plain text / clipboard
2. Markdown / TXT / subtitles
3. CSV
4. Office documents
5. EPUB and other structured formats

## Platform direction

Long-term target:

- Windows
- macOS
- Android
- iOS / iPadOS

Desktop, CLI and MCP must reuse one HanContext Core / Runtime: same rules, same
data, same result. Preserve Runtime API v1 unless separately approved.

Suggested order: Desktop Alpha → product CLI → MCP Server → production Windows/macOS
packaging/signing → editor/plugin integrations → optional AI assistance.
Alpha packaging/onboarding means limited development distribution, not production signing.

A product CLI may support batch localization, scripting, automation, CI and developer
workflows. The current Rust demo CLI already uses Runtime API v1; productizing it is
future work. A future MCP server may expose localization, terminology inspection,
alternatives and explanations through the same runtime. Neither reimplements the engine.
