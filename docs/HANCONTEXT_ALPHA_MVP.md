# HanContext Alpha MVP — planning specification

Date: 2026-10-08. Status: proposed; separate approval is required for implementation.

**HanContext — Chinese, localized with context. / 懂情境的中文地區化**.
Use **Chinese Mainland**, Hong Kong and Taiwan in future product copy.

## Goal and entry gate

Prove a usable, explainable, reproducible plain-text workflow using the shared
deterministic runtime. Finish/review Phase 3J / Issue #38 first
([PR #45](https://github.com/168aadc852/chinese-regional-localizer/pull/45)). Keep every
major Alpha feature in a separate approved, tested PR; split packages further if needed.

Current routes: `zh-CN → zh-HK/zh-TW` and `zh-Hant → zh-HK/zh-TW`. Source-region
labels must map honestly to supported routes. Do not imply all regional pairs or
conversion to `zh-CN` already work. Extra routes require a separate issue.
Preserve Runtime API v1 unless a specific issue approves a version/migration change.

## Intended experience

Input → source region → target region → usage context → localization behaviour
→ localize → review result → inspect important changes → choose valid alternatives
→ optionally remember a preference.

Main UI uses plain language and hides database priority, rule IDs, confidence scores,
provenance schemas and raw tables. Explain why wording changed first. Technical
source/record/rule/data-version details are available only on demand.

Proposed modes: Script only, Standard localization (default), Conservative localization.
Conservative leaves ambiguous wording unchanged; no “Full is better” framing.
Approve exact semantics and tests before implementing modes.

## Context and preference requirements

Possible built-ins: General; Technology / Software; Banking / Finance; Business /
Marketing; Legal; Education; Government / Public Administration. These are proposed
profiles, not promises of terminology coverage. Existing licence/ingestion gates apply.

Custom profiles such as Hi-Fi Audio, ESG and University Administration should inherit
a general context. Define stable identity, parent relationships, cycle rejection,
missing-parent behavior, inheritance ranking and migrations before implementation.
Do not duplicate every rule or require LLM context classification.

My Terms preferences conceptually depend on source term, preferred term, target locale
and context, not a permanent global replacement. Explain them as “Use X instead of Y
when writing Hong Kong Chinese for Technology / Software.” Keep priorities internal.
Preserve private dictionaries through an explicit compatibility/migration plan;
desktop settings v1 is not the user-term/context schema.

Proposed effective precedence:

1. Exact-context user preference.
2. All-context user preference.
3. Governed current-context terminology.
4. General target-region terminology.
5. Script / character conversion.

Reconcile inherited-context ranking, protected terms, entities and longest-match rules
with the existing contract before changing behavior. Equal-level ties stay unresolved;
preserve original text rather than guess. Use Recommended / Also valid / Needs your decision.

Alternative selection updates a specific occurrence without rerunning the document.
Define stable span tracking and undo so adjacent edits remain correct. Offer Use this
time only / Remember for this context / Remember for all contexts.

## Reviewable work packages

| Package | Scope | Dependencies |
| --- | --- | --- |
| [#46 Context profiles/model](https://github.com/168aadc852/chinese-regional-localizer/issues/46) | Versioned identity, inheritance and migration design | Phase 3J reviewed; model approval |
| [#47 Context-aware selection](https://github.com/168aadc852/chinese-regional-localizer/issues/47) | Governed deterministic selection and inheritance | #46; approved behavior contract |
| [#48 Alternatives/ambiguity](https://github.com/168aadc852/chinese-regional-localizer/issues/48) | Candidates, ties, spans and local alternative application | #47 |
| [#49 My Terms](https://github.com/168aadc852/chinese-regional-localizer/issues/49) | Context/all-context preferences and private-dictionary migration | #46–#48 |
| [#50 Alpha desktop UX](https://github.com/168aadc852/chinese-regional-localizer/issues/50) | Plain-text workflow, modes, explanations and preference controls | #46–#49; mode semantics approval |
| [#51 Alpha packaging/onboarding](https://github.com/168aadc852/chinese-regional-localizer/issues/51) | Limited development distribution and offline first-use guidance | #50; fixture vs release-data distinction |

Issue creation records proposals/dependencies, not blanket implementation approval.

## Acceptance direction

- Supported routes complete the workflow without an LLM or external text upload.
- Identical input/settings/context/data-version combinations produce identical output.
- Profile inheritance and locale/context preferences have regression tests.
- Equal-level ties preserve text and expose decisions without arbitrary selection.
- Alternatives affect only the intended occurrence, without stale spans/full reruns.
- Remembered preferences respect the chosen context/all-context scope and target locale.
- Important changes have plain-language explanations and optional deeper provenance.
- Missing/corrupt data/preferences fail safely and preserve approved API compatibility.
- Windows beginner onboarding labels Alpha limitations and data licences clearly.

## Later sequence and exclusions

Desktop Alpha → product CLI → MCP Server → production Windows/macOS packaging/signing
→ editor/plugin integrations → optional AI assistance. Desktop, CLI and MCP call the
same Core / Runtime; cross-client parity tests are required when clients are added.
Future MCP capabilities may include localization, terminology, alternatives and explanations.

No repository rename, production signing, background scheduling, broad data ingestion,
mobile UI, file-format pipeline or AI implementation belongs in Phase 3J or this
planning change. Release hosting/retry-resume/key operations remain separate follow-ups
and may become prerequisites for later distribution.
