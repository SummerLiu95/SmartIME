# AGENTS.md

This file is the AI code agent entrypoint for SmartIME. It intentionally acts as an index instead of duplicating project requirements, developer commands, architecture notes, or release lessons.

When working in this repository, read only the document(s) that are relevant to the current task. Do not load every project document by default. After identifying the relevant owner document, keep updates in that owning document.

## Canonical Documentation

| Need | Read / Update |
| :--- | :--- |
| Developer setup, commands, local LLM config, debugging, release workflow, and user/developer-visible Milestone roadmap | `README.md` |
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
- If the user manually records or asks to adjust the public feature roadmap, update only the `Milestone` section of `README.md`; detailed behavior and acceptance criteria still belong in `docs/REQUIREMENTS.md` after feasibility confirmation.
- If release history changes, update `CHANGELOG.md`.

## Project Workflow

1.  **Product iteration intake**
    The user records product feature iteration ideas in the `Milestone` section of `README.md`. This section is a public roadmap for developers and app users, not the owner of detailed requirements.

2.  **Technical feasibility research**
    When the user is ready to develop a milestone item, first produce a technical feasibility research report for manual confirmation. Do not expand the requirement into `docs/REQUIREMENTS.md` or `docs/DESIGN_DOC.md` before the user confirms the feature is technically feasible.

3.  **Requirement and design expansion after feasibility confirmation**
    After the user confirms the feature is technically feasible, update the relevant context documents:
    - Expand product behavior, constraints, and acceptance criteria in `docs/REQUIREMENTS.md`.
    - Update `docs/DESIGN_DOC.md` when the feature affects UX, interaction design, visual behavior, or when the user provides design resources.
    - If the user provides a Figma design or other design reference, record the relevant link, node, screen, or design intent in `docs/DESIGN_DOC.md`.

4.  **Planning**
    Update `docs/TASKS.md` only in these two cases:
    - the user explicitly asks AI to plan the requirement implementation
    - work is being planned in Plan mode

5.  **Confirmed implementation**
    After the user confirms a plan from `docs/TASKS.md`, proceed with code development according to the confirmed plan.

6.  **Mistakes and special testing lessons**
    During development, if AI makes a clear mistake that causes behavior to diverge from expectations, or if the feature requires special test preparation, record the lesson in `docs/Rulebook.md` so future AI agents do not repeat it.

7.  **Completion record**
    After development is complete, create a focused implementation record under `docs/exec-plan/` summarizing what was built, affected files/modules, validation performed, and any follow-up notes.

8.  **Architecture updates**
    Update `docs/TECHNICAL_SPEC.md` only when the actual code changes affect architecture, module responsibilities, IPC contracts, runtime state, persistence layout, build/deployment design, or other technical system design.

## Agent Workflow

1.  Start with this index to find the owning document.
2.  Read only the relevant canonical docs before editing code or documentation.
3.  Keep changes small and aligned with existing project patterns.
4.  Follow the project workflow above for requirement expansion, planning, implementation records, and architecture updates.
5.  Apply the AI mistake-prevention notes and testing methods from `docs/Rulebook.md` for macOS/Tauri behavior.
