# D-011 — Source refreshes preserve history but expose one current resource snapshot

Date: 2026-10-07

External data is updateable, so append-only imports without a current/superseded concept are unsafe.

Decision:

- each independently refreshable upstream resource has a stable `resource_key`;
- `source_versions.is_current` identifies the runtime snapshot for that resource;
- identical revision/checksum re-imports are idempotent;
- a changed snapshot marks the previous resource version non-current rather than deleting history;
- source-specific active rows (for example OpenCC term rules) are deactivated when superseded;
- evidence-backed runtime lookups consider only current source versions;
- pack and manifest metadata are refreshed/enforced during import;
- SQLite v0.1 remains historical; migration 0003 upgrades existing databases to the v0.2 lifecycle.

This keeps provenance auditable while preventing stale labels/rules from competing with current upstream data.
