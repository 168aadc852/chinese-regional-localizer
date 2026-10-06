# D-010 — AI context is loaded on demand

Date: 2026-10-07

AI entry instructions must remain small. Agents should start from the current task/Issue and load only task-relevant files through `AGENTS.md` and `.ai/` guides.

Rules:
- do not preload README, project history or the documentation tree;
- prefer specific files/sections over broad context;
- `PROJECT_STATE.md` contains current status only;
- historical detail belongs in `docs/history/` and Git/Issue history;
- architecture decisions are split into individual ADR files;
- tool-specific instruction files stay thin and defer to `AGENTS.md`.

Reason: reduce repeated context/token cost while preserving durable project knowledge and safe handoff between AI tools.
