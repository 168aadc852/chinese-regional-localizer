# Deterministic Localization Engine — Phase 2A

Phase 2A is the first end-to-end text localization layer. It is a **Python reference implementation** used to lock behavior before the future production Rust runtime is built.

## Design principle

The engine is deterministic and offline. It does not use an LLM.

Processing order:

1. identify known entities in the source locale;
2. resolve them to the target regional name when unambiguous;
3. protect resolved entity spans;
4. apply staged generic/script/regional terminology rules using longest-match scanning;
5. return output plus structured explanations/provenance;
6. flag ambiguous cases for human review instead of guessing.

## Initial routes

Supported Phase 2A routes:

- `zh-CN -> zh-HK`
  - entity resolution directly to `zh-HK`;
  - `zh-CN -> zh-Hant` term stage;
  - `zh-Hant -> zh-HK` regional stage.

- `zh-CN -> zh-TW`
  - entity resolution directly to `zh-TW`;
  - `zh-CN -> zh-Hant` term stage;
  - `zh-Hant -> zh-TW` regional stage.

- `zh-Hant -> zh-HK`
- `zh-Hant -> zh-TW`

More routes can be added only when their data semantics are explicit.

## Entity priority

Entity names and aliases are indexed from `localized_names` for the declared source locale.

At each position the engine uses **longest-match-first**.

If exactly one concept matches and exactly one preferred target-regional name exists, that regional entity name is used.

The replaced span is then protected from later generic term conversion. This prevents a correct entity such as a Hong Kong personal name from being corrupted by a generic dictionary rule.

### Entity ambiguity

If one surface form matches several concepts, the engine does not guess.

It:

- leaves the source text unchanged;
- protects that span from generic conversion;
- returns `review_needed: true`;
- returns the candidate concepts/QIDs where available.

Likewise, if a known entity has no target-regional preferred name, or has several conflicting preferred target names, Phase 2A leaves the span unchanged and asks for review.

## Term-rule matching

Rules come from `term_rules` and are applied stage by stage.

Matching policy:

1. longest source phrase first;
2. among rows for that exact source phrase, highest priority first;
3. lower-priority target candidates are retained as alternatives;
4. if the highest priority is tied between different target strings, do not guess.

A tie produces an unchanged, protected span with `review_needed: true`.

The implementation never performs repeated global `str.replace` operations.

## OpenCC candidate preservation

The Phase 1B importer preserved OpenCC source candidate ordering. The reference engine uses the highest-priority candidate in Phase 2A but exposes alternatives in the explanation structure.

This policy is deliberately reversible: future context rules can choose a non-default candidate without rebuilding the source data.

## Explainability output

`LocalizerEngine.localize()` returns a dictionary containing:

- original input;
- final output;
- source/target locales;
- route stages;
- list of change/review events;
- overall `review_needed` flag.

Entity events may include:

- original/replacement text;
- source and output spans;
- concept ID/type;
- Wikidata QID;
- confidence;
- name-evidence source, revision, URL, checksum and retrieval time.

Term-rule events may include:

- stage;
- original/replacement text;
- stage-local spans;
- source ID;
- source-version ID;
- revision/version label;
- upstream dictionary record/URL;
- checksum;
- candidate alternatives.

Phase 2A term-rule spans are stage-local, because an earlier stage may have changed text length. A later UI/diff layer can add full original-to-final alignment separately.

## Evaluation corpus

`data/fixtures/evaluation_cases.json` currently tests:

- mainland Brad Pitt name -> Hong Kong name;
- mainland Brad Pitt name -> Taiwan name;
- `人工智能 -> 人工智慧` for Taiwan;
- Simplified/Traditional phrase conversion;
- a sentence combining entity and terminology conversion.

Automated tests additionally inject conflict cases to verify:

- entity replacement protection;
- longest phrase wins over a shorter rule even if the shorter rule has higher priority;
- same-priority conflicting rules trigger review;
- identical source names mapped to multiple entities trigger review.

## Build the development demo database

From the repository root:

```bash
python scripts/build_demo_database.py
```

This creates:

```text
build/regional-demo.sqlite
```

Important: this database uses deliberately small repository fixtures and is **not** an authoritative public data release.

## Try the localizer

Example Hong Kong person localization:

```bash
python scripts/localize_text.py \
  --db build/regional-demo.sqlite \
  --from zh-CN \
  --to zh-HK \
  --text "布拉德·皮特"
```

Example Taiwan mixed sentence:

```bash
python scripts/localize_text.py \
  --db build/regional-demo.sqlite \
  --from zh-CN \
  --to zh-TW \
  --text "布拉德·皮特研究人工智能"
```

Expected fixture result:

```text
布萊德·彼特研究人工智慧
```

The JSON output also explains which part was an entity change and which part came from an OpenCC rule.

## Production direction

The Python engine is a behavior/reference implementation, not the final application runtime.

Before moving to the future Rust core, Phase 2 should first stabilize:

- matching semantics;
- conflict resolution;
- provenance/explanation format;
- user/protected term precedence;
- evaluation corpus and regression tests.

The Rust implementation should reproduce these tested behaviors rather than redesigning them implicitly.
