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
| 6 | OpenAlex harvest ≥1000 works → research/corpus.jsonl + SEARCH-LOG.md | done | 2793 unique works; commit 5868a3d; anchors Mayer2003/Karpicke2008/Cepeda2006 | curation pass later |
| 7 | bun install && make ci green | done | nix develop -c make ci: typecheck 0 errors, tests 1/1; first-run hang was cold-start contention, clean on warm env |; |
| 8 | git init + gh repo create --public + first push | done | github.com/WyattAu/edureel public; af26a4b, 5868a3d pushed | keep pushing per flap rule |
| 9 | Register in engineering-standards estate.yml | done | 7cc3289 pushed; YAML validated |; |
| 10 | Product brief + capability contract | done | PRODUCT-BRIEF.md, docs/product/CAPABILITY.md; commit 468297a |; |
| 11 | S1+S2: crates/srs (Tachyon port) + crates/topics (graphkit) | active | subagent dispatched (general) | verify cargo test |
| 12 | content/ seed data (topics.json, clips.json, questions.json) | active | inline while subagent runs | CC-BY original micro-lessons, license-unverified externals excluded |
| 13 | S3: web feed + recall gate + review tab | blocked | blocked by #11 API + #12 data | dispatch after |

## Dogfood inventory (decided)
- graphkit (Rust) — topic prerequisite DAG. Tachyon flashcards/srs.rs — simplified FSRS (extract/PR). web-estate constitution — TS+SolidJS+Bun, @est/* kits. feedkit — partial (adapters). License: Apache-2.0 (estate default, carried by template).

## Next action
Run scripts/harvest_openalex.py; verify corpus.jsonl ≥1000 unique works.

## Log
- 2026-11-21 rev1-2 recon complete; server restart; ledger to /tmp
- 2026-11-22 rev3 copier render + renames + AGENTS.md invariants; ledger moved into repo
- 2026-11-22 rev4 corpus 2793 works (5868a3d); estate registration (7cc3289); make ci green under nix bun 1.4.4; bootstrap complete
