# AGENTS.md — instructions for AI coding agents

Project: **edureel** — short-form video learning platform (public, Apache-2.0).

## Non-negotiable contract

1. **Scripts are canonical.** `scripts/*` do the work; `make` wraps them;
   CI runs the same scripts. NEVER add CI-only steps.
2. **One source of truth per concern.** Version pins, lint policy, and
   formatting each live in exactly one file. Never add a parallel config.
3. **Never weaken the gates.** Coverage thresholds, lints, and policy
   checks are lowered only via an ADR (`docs/adr/`) with justification.
4. **Never commit secrets.** Placeholders in `.env.example` style only.
5. **Decisions get ADRs.** Layout, gate, or tooling changes require
   `docs/adr/NNNN-*.md` (Status/Context/Decision/Consequences).

## Stack

- **Web:** bun workspaces — Astro shell + SolidJS islands (`apps/web`),
  shared TS packages under `packages/` (`@edureel/*` scope).
- **Learning engine (planned):** Rust crates under `crates/` —
  FSRS-style scheduler and topic prerequisite graph, dogfooding estate
  `graphkit`; SRS derived from Tachyon `flashcards/srs.rs` (extract/PR,
  never fork).
- **Toolchain:** nix flake owns system tools; bun owns JS packages.
- **Estate:** registered in `engineering-standards/estate.yml`; license
  Apache-2.0; conformance fixtures under `conformance/` when engines land.

## Product invariants (edureel-specific)

6. **Learning gain outranks engagement** in every ranking decision.
   A mechanic that raises watch time but not measured recall is a bug.
7. **Mechanics cite research.** Every learning mechanic ships with a
   pointer into `research/` (paper ID + claim) before or with the PR.
8. **No dark patterns.** No infinite-scroll-without-recall, streak guilt,
   or engagement triggers that research shows harm retention/transfer.
   Violations are ADR-worthy regressions, not style nits.

## Commands

$3

- `make bench` (perf budget vs committed baseline) / `make bench-update`

- `make repro` (build twice, compare artifact hashes)

## Conventions

- Commits: conventional prefixes (`feat:`, `fix:`, `test:`, `docs:`, `chore:`).
- Requirements are REQ-tagged and traced in `REQUIREMENTS.md`.
- Estate-wide standards: [engineering-standards](https://github.com/WyattAu/engineering-standards)
  and [OMNI-CORE.md](https://github.com/WyattAu/engineering-standards/blob/main/OMNI-CORE.md).
- When unsure, read `ARCHITECTURE.md` — it explains *why*, this file
  explains *what*.
