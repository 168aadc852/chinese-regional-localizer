# Project Overview

## Problem

Most Chinese conversion tools focus on Simplified vs Traditional characters. Real regional localization is broader: Mainland China, Hong Kong and Taiwan often use different terminology, proper-name translations, media titles, person names, institutional terms and writing conventions.

## Proposed product

A local, open-source Chinese regional localization assistant supporting:

- Mainland China Chinese (`zh-CN`)
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
- Confidence/review workflow rather than blind replacement.
- Community-maintained corrections with source tracking.
- Data packs that can be updated separately from the application.

## Non-goals for the first implementation

- Full Cantonese ↔ Mandarin translation.
- General-purpose machine translation between Chinese and non-Chinese languages.
- Replacing source-specific legal or professional dictionaries.
- Using an LLM for every conversion.
