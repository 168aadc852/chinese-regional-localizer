# Project Decisions

## D-001 — GitHub is the project source of truth

Date: 2026-10-06

Decision: Repository files and Issues will hold durable project context so different AI tools and human contributors can continue work without relying on a specific chat history.

## D-002 — Data governance precedes application development

Date: 2026-10-06

Decision: Build and verify the data-source whitelist, policy and schema before significant UI/application work.

Reason: Licensing and provenance are foundational to an open-source terminology/localization product.

## D-003 — Offline-first product direction

Date: 2026-10-06

Decision: Normal localization should be performed locally. Internet access is primarily for explicit database/application updates.

## D-004 — Three regional targets are first-class

Date: 2026-10-06

Decision: `zh-CN`, `zh-HK` and `zh-TW` are distinct targets. The project is not merely a Simplified/Traditional converter.

## D-005 — Separate code licences from data licences

Date: 2026-10-06

Decision: Do not apply one blanket repository licence to third-party datasets. Maintain separable data layers/packs where obligations differ.

## D-006 — AI agents must write back important context

Date: 2026-10-06

Decision: Significant findings, decisions and progress must be committed to repository documentation, not left only in AI chat history.

## D-007 — Current technical direction is cross-platform but not locked

Date: 2026-10-06

Direction: Tauri 2 + Rust core + SQLite is the leading architecture candidate for desktop/mobile reuse. This remains subject to proof-of-concept validation before implementation lock-in.
