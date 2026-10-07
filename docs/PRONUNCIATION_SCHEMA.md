# Pronunciation Schema — Phase 2C

Pronunciation data is added by `schema/migrations/0002_pronunciations.sql` and participates in the v0.2 source-version lifecycle.

`pronunciations` stores concept, locale, romanization scheme, reading components, source/version identity, upstream record/URL and confidence. Multiple readings for one character are valid.

Migration `0003_core_hardening.sql` adds the `current_pronunciations` view, which excludes rows belonging only to superseded source versions.

The LSHK importer uses resource key `lshk:list.tsv`, verifies the pinned production Git blob, records SHA-256 of imported bytes, and makes repeat imports of the same revision idempotent.
