# edureel — bootstrap + literature harvest

status: active
updated: 2026-11-22  rev: 3

## State
| # | Task | Status | Evidence | Next |
|---|------|--------|----------|------|
| 1 | Skills loaded (eng-standards, estate-first, openalex, plan-ledger) | done | session |; |
| 2 | Recon + dogfood scan | done | `gh auth status`; tree=226 repos |; |
| 3 | Decide name + stack | done | user: edureel, TS web + Rust core |; |
| 4 | Derive from OmniTS-template via copier | done | `~/dev/src/github.com/WyattAu/edureel` rendered |; |
| 5 | Rename workspaces + fill AGENTS.md | done | pkg names `edureel`/`@edureel/web`; invariants 6-8 |; |
| 6 | OpenAlex harvest ≥1000 works → research/corpus.jsonl + SEARCH-LOG.md | active | root cause: cursor paging key-gated (401 body); offset `page=N` works keyless; rerun sh_200b3d54 | await completion |
| 7 | bun install && make ci green | blocked | blocked by #6 (same session) | run after harvest |
| 8 | git init + gh repo create --public + first push | blocked | blocked by #7 | push immediately (flap rule) |
| 9 | Register in engineering-standards estate.yml | blocked | blocked by #8 | area=app, status=personal→? ask |

## Dogfood inventory (decided)
- graphkit (Rust) — topic prerequisite DAG. Tachyon flashcards/srs.rs — simplified FSRS (extract/PR). web-estate constitution — TS+SolidJS+Bun, @est/* kits. feedkit — partial (adapters). License: Apache-2.0 (estate default, carried by template).

## Next action
Run scripts/harvest_openalex.py; verify corpus.jsonl ≥1000 unique works.

## Log
- 2026-11-21 rev1-2 recon complete; server restart; ledger to /tmp
- 2026-11-22 rev3 copier render + renames + AGENTS.md invariants; ledger moved into repo
