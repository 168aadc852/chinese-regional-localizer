# D-025 — Versioned private context preferences and request-bound remembering

Date: 2026-10-10. Accepted by Issue #49 Design Approval v0.1 and Implementation Brief v0.1.

## Decision

- Keep private SQLite separate from shared data and desktop settings. Add v2 dictionary
  containers and the stable reserved `personal`/`legacy` identities.
- Migrate all historical fields and IDs transactionally; preserve null scopes and
  legacy specificity/priority, never guess a target or context. Failures retain v0.1.
- Keep longest eligible private phrase and protection before ordinary preferences;
  rank exact, nearest inherited ancestor, then all-context. At the same effective
  level deduplicate target text and preserve conflicts, never choose by dictionary order.
- Retain legacy specificity/priority only within a wholly legacy level. New record
  conflicts cannot be resolved by exposing or importing numeric priority controls.
- Use #48's owned occurrence/revision/span/candidate/undo state for remembering.
  A request-bound wrapper freezes original input/locales/context, saves the original
  source interval and explicitly rejects partial original-source expansions.
- Commit Personal upserts atomically, target-locale scoped. Explicit newer choices
  update their exact Personal scope only. Text-choice success and persistence failure
  are separately reported; undo is text-only, not deletion of remembered data.
- Keep CSV preview and transaction commit in the private core; no third-party content,
  filesystem paths in user responses or frontend storage/selection logic.

## Compatibility / consequence

Runtime API v1 keeps its outer fields, optional nested metadata only. Shared schemas,
routes, entity/script/regional pipeline, profile validation and settings v1 stay intact.
Private v2 is not backward-writable by software that blindly re-executes the historical
v0.1 schema; use version-dispatched open. Keep an external backup before downgrading software.
No polished #50 UX, cloud/team dictionaries, AI or packaging changes are approved here.

Canonical contract, migration details, operations and test fixtures:
[My Dictionaries](../MY_DICTIONARIES.md).
