# User Dictionary

Phase 2D adds a private local customization database that is deliberately separate from shared/open-data packs.

Issue #49 evolves this foundation to versioned My Dictionaries/My Terms. See
[the current v2 contract](MY_DICTIONARIES.md) for transactional legacy migration,
dictionary activation, context-aware preference ranking and remembered-choice writes.
The examples below remain the legacy/default dictionary compatibility commands;
they are not the new Alpha dictionary-manager UI.

## Why it is separate

`user_dictionary.sqlite` belongs to the local user. It is not redistributed, is ignored by Git, and is not subject to the source-pack licensing model used for OpenCC/Wikidata/etc.

## Rule types

### Protected term

Preserve the exact source text and do not pass that span through entity, terminology or script conversion.

Example:

```bash
python scripts/user_dictionary.py protect OpenAI
```

### Fixed override

Replace an exact source surface with the user's chosen target text, then protect the replacement from later shared rules.

Example:

```bash
python scripts/user_dictionary.py override 數位 數碼 --target-locale zh-HK
```

Rules can optionally be scoped by `--source-locale` and/or `--target-locale`.

## Default database location

The management CLI defaults to:

```text
~/.chinese-regional-localizer/user_dictionary.sqlite
```

You can supply another path with `--db`.

## Management commands

```bash
python scripts/user_dictionary.py protect PERLISTEN
python scripts/user_dictionary.py override 數位 數碼 --target-locale zh-HK
python scripts/user_dictionary.py list
python scripts/user_dictionary.py disable 2
python scripts/user_dictionary.py enable 2
python scripts/user_dictionary.py remove 2
```

## Localizing with the private dictionary

```bash
python scripts/localize_text.py \
  --db build/localizer-demo.sqlite \
  --user-db ~/.chinese-regional-localizer/user_dictionary.sqlite \
  --from zh-CN \
  --to zh-HK \
  --text "OpenAI和布拉德·皮特"
```

Without `--user-db`, behavior remains the existing shared deterministic engine.

## Precedence

1. protected user term;
2. user fixed override;
3. shared entity localization;
4. shared regional terminology;
5. generic/script rules.

See ADR D-012 for legacy behavior and [D-025](adr/D-025-versioned-private-context-preferences.md)
for v2 context preferences; #47 shared stages/routes remain unchanged.
