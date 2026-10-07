# Deterministic Localization Engine — Phase 2C

The Python engine is the behavior/reference implementation used before the production Rust runtime.

## Processing order

1. Build/reuse cached match indexes for the requested locales.
2. Match only conservative, high-confidence entity surfaces.
3. Resolve one unambiguous current target name or flag review; protect the span.
4. Apply staged terminology/script rules using trie-based longest-match scanning.
5. Enforce rule constraints when a `constraints` object is present.
6. Preserve original-to-final alignment and provenance for every change/review event.

## Routes

- `zh-CN -> zh-HK`: entity resolution, `zh-CN -> zh-Hant`, then `zh-Hant -> zh-HK`.
- `zh-CN -> zh-TW`: entity resolution, `zh-CN -> zh-Hant`, then `zh-Hant -> zh-TW`.
- `zh-Hant -> zh-HK`.
- `zh-Hant -> zh-TW`.

## Entity safety

Wikidata-scale entity data creates lexical collisions. The reference engine therefore does not auto-interpret very short/common surfaces as entities merely because they exist in the entity database.

Current conservative policy:

- pure-CJK preferred surfaces normally require at least 3 characters;
- pure-CJK aliases normally require at least 4;
- punctuation/mixed-script names may be shorter;
- ASCII surfaces require token boundaries;
- only names with current evidence (or test/manual names with no evidence rows) participate.

This intentionally prefers a missed automatic entity conversion over corrupting ordinary prose. Future contextual NER may expand coverage only with measured regression tests.

## Snapshot freshness

Names supported only by superseded `source_versions` do not participate in runtime matching. This prevents a Wikidata rename from leaving old and new labels simultaneously preferred at runtime.

## Rule precedence and constraints

For each stage:

1. longest applicable source phrase wins;
2. for that phrase, highest priority wins;
3. lower-priority targets remain alternatives;
4. tied highest-priority different targets trigger review.

`context_constraint` JSON is no longer merely decorative. Executable restrictions are placed under `constraints`, currently supporting:

- `domain`;
- `preceded_by` / `not_preceded_by`;
- `followed_by` / `not_followed_by`;
- `word_boundary`.

Malformed constraint JSON is not silently treated as unrestricted.

## Explainability and alignment

Every entity/term event carries its original input span. After all stages, the engine adds `final_output_span`, allowing a UI to highlight the final output even when earlier stages changed text length.

Existing stage-local span fields remain for debugging/provenance compatibility.

## Performance model

Entity and term indexes are compiled into prefix tries once per `LocalizerEngine` instance and cached by locale/stage instead of being rebuilt on every call. Long-running desktop/mobile runtimes should reuse one engine instance for a static database and call `clear_cache()` after database mutation.

The future Rust implementation may use a more compact automaton, but must reproduce tested behavior rather than silently changing precedence.

## OpenCC limitation

The term layer preserves reviewed staged dictionaries, longest matching and dictionary/candidate precedence, but is still not a complete OpenCC runtime implementation. Generated segmentation resources, all mmseg details, reverse configs and plugins remain explicit future work.
