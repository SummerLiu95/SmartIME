# AGENTS.md

This file is the AI code agent entrypoint for SmartIME. It intentionally acts as an index instead of duplicating project requirements, developer commands, architecture notes, or release lessons.

When working in this repository, read only the document(s) that are relevant to the current task. Do not load every project document by default. After identifying the relevant owner document, keep updates in that owning document.

## Canonical Documentation

| Need | Read / Update |
| :--- | :--- |
| Developer setup, commands, local LLM config, debugging, release workflow | `README.md` |
| Product behavior, UX constraints, and acceptance criteria | `docs/REQUIREMENTS.md` |
| Product and interaction design | `docs/DESIGN_DOC.md` |
| Technical architecture, module responsibilities, IPC contracts, runtime state, build design | `docs/TECHNICAL_SPEC.md` |
| User-requested or Plan-mode task planning and dependency sequencing | `docs/TASKS.md` |
| AI-prone mistakes, testing methods, project lessons that prevent repeated errors | `docs/Rulebook.md` |
| Execution records created after a confirmed `docs/TASKS.md` plan is implemented | `docs/exec-plan/` |
| User-facing release history | `CHANGELOG.md` |

## Documentation Ownership

- Do not duplicate substantial content from the canonical docs into this file.
- If product behavior changes, update `docs/REQUIREMENTS.md`.
- If architecture or implementation design changes, update `docs/TECHNICAL_SPEC.md`.
- Update `docs/TASKS.md` only when the user explicitly asks AI to plan tasks, or when work is being planned in Plan mode.
- After a `docs/TASKS.md` plan from one of those two scenarios is confirmed and executed, add or update a focused record under `docs/exec-plan/`.
- If an AI-prone mistake, regression test method, or repeatable project lesson changes, update `docs/Rulebook.md`.
- If developer commands or setup steps change, update `README.md`.
- If release history changes, update `CHANGELOG.md`.

## Agent Workflow

1.  Start with this index to find the owning document.
2.  Read only the relevant canonical docs before editing code or documentation.
3.  Keep changes small and aligned with existing project patterns.
4.  Use `docs/TASKS.md` only when the user explicitly asks for task planning, or when operating in Plan mode.
5.  After one of those plans is confirmed and executed, record the completed implementation under `docs/exec-plan/`.
6.  Apply the AI mistake-prevention notes and testing methods from `docs/Rulebook.md` for macOS/Tauri behavior.
