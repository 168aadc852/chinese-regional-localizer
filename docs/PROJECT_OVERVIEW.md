# Project Overview

## Problem

Most Chinese conversion tools focus on Simplified vs Traditional characters. Real regional localization is broader: the Chinese Mainland, Hong Kong and Taiwan often use different terminology, proper-name translations, media titles, person names, institutional terms and writing conventions.

## Proposed product

A local, open-source tool with working identity **HanContext — Chinese, localized
with context. / 懂情境的中文地區化**, intended to become controllable, explainable,
deterministic and context-aware across:

- Chinese Mainland Chinese (`zh-CN`)
- Hong Kong Chinese (`zh-HK`)
- Taiwan Chinese (`zh-TW`)

The tool should convert text while distinguishing several classes of change:

1. character/script conversion;
2. regional terminology;
3. proper-name/entity localization;
4. user/community overrides;
5. uncertain changes requiring review.

## Intended differentiators

- Offline-first processing.
- Multi-source, traceable terminology/entity database.
- Film/person/organisation and other proper-name localization.
- Explanations and provenance for important changes.
- Plain-language ambiguity/review workflow, with technical provenance in a deeper optional layer.
- Community-maintained corrections with source tracking.
- Data packs that can be updated separately from the application.

## Non-goals for the first implementation

- Full Cantonese ↔ Mandarin translation.
- General-purpose machine translation between Chinese and non-Chinese languages.
- Replacing source-specific legal or professional dictionaries.
- Using an LLM for every conversion.

## Next development gate

Phase 3J / Issue #38 settings persistence is complete. The consolidated direction is
in `PRODUCT_REQUIREMENTS.md`; `HANCONTEXT_ALPHA_MVP.md` scopes proposed Alpha work.
These describe future capabilities, not implemented features. Next review and separately
approve the first scoped Alpha issue, beginning with #46. Desktop, the future product
CLI and MCP server must reuse the same Core / Runtime. No repository rename or Alpha
implementation is part of this planning update.
