# CAPABILITY — edureel MVP loop (v0)

CAPABILITY
A self-learner picks the CS-fundamentals track and gets a vertical feed of
seeded clips where each clip is (a) prerelease-gated by the topic
prerequisite graph and (b) followed by one recall question; answering
schedules the clip's concept into an FSRS-style review queue, and a
Review tab resurfaces clips at their computed due dates. After v0 ships,
a learner's next-swipe content is determined by mastery state, not
novelty.

CONSTRAINTS (fixed policy — from AGENTS.md invariants + PRODUCT-BRIEF)
- INV-6: ranking/sequencing must optimize demonstrated recall, never
  watch time. No engagement metric enters any ordering decision.
- INV-7: every mechanic PR cites an anchor work (see brief mechanic map).
- INV-8: no infinite scroll without recall; no streak/guilt mechanics.
- Recall questions are hand-authored in v0; never generated.
- Seed clips only with recorded license (CC-BY/CC0/public/OER-verified);
  NC and unknown licenses excluded.
- All state local/first-party in v0: no accounts, no social graph, no
  third-party analytics.

IMPLEMENTATION CONTRACT
Actors: learner (sole v0 actor); content curator (repo editor).

Surfaces:
- `crates/srs` — Rust: FSRS-style scheduler ported from WyattAu/Tachyon
  `flashcards/srs.rs` (learning steps → review with stability intervals;
  adapt, do not fork-and-diverge semantics silently). Pure, no IO.
  API: schedule(state, rating, now) -> state'; due(state) -> bool.
- `crates/topics` — Rust: prerequisite DAG over topic nodes consuming
  estate `graphkit` (typed link graph, n-hop queries). API: unlock(topic,
  mastered_set) -> bool; frontier(mastered_set) -> [topic]. Seeded with
  the CS-fundamentals topic set from content/topics.json.
- `apps/web` — SolidJS islands: vertical feed (scroll-snap), recall gate
  modal, Review tab, track view. Reads topics/srs via a TS store seeded
  from JSON; wasm binding for crates/srs is slice 2, not v0-blocking.
- `content/` — versioned seed data: topics.json (graph), clips.json
  (id, topic, duration, license, source url), questions.json (clip id →
  question, options, answer, misconception notes).

States and transitions:
- Topic: locked → unlocked → in_progress → mastered
  (mastered = concept's review card at Review-stage stability ≥ threshold).
- Card (per concept): New → Learning(step 0..n) → Review → lapse→Learning.
- Session: feed view interleaves due reviews first, then unlocked-frontier
  new topics (INV-6 order: due > frontier-new > nothing; no filler).

Interfaces:
- crates/* publish no crate in v0 (workspace-local; publish is post-MVP).
- content/*.json are the only content surface; app never hardcodes clips.
- grading: rating ∈ {again, hard, good, easy} per card, mapped from
  recall answer correctness (again) vs latency-free correctness (others).

Data model implications: single local store (localStorage/IndexedDB in
web; JSON export/import path required for later server move).

Observability: v0 local-only event log (events: clip_start, recall_answer,
review_grade, session_end) written to the local store, exportable — this
is the future 7-day-recall instrument (PRODUCT-BRIEF working metric).

NON-GOALS (v0)
- No creator upload pipeline, follows, comments, likes, DMs.
- No server, accounts, sync, push, notifications.
- No auto-generated questions; no LLM in the loop.
- No generalization beyond the CS-fundamentals track.
- No native mobile.

OPEN QUESTIONS (do not block v0; log decisions when resolved)
- Q1 review-tab clip replay vs card-only quiz: default card-only quiz.
- Q2 lapse policy: default 1 lapse → Learning step 0 (Tachyon parity).
- Q3 wasm vs TS shim for srs in web: default TS shim v0, wasm slice 2.

HANDOFF
Ready for direct implementation as three slices in order:
S1 crates/srs (+tests, cargo green), S2 crates/topics on graphkit
(+seeded graph, tests), S3 web feed consuming content/ + engine API
(make ci green end to end). Lane: tdd-workflow per slice; verification
loop before final push.
