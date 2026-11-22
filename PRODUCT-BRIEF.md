# PRODUCT-BRIEF — edureel

status: draft v1 (2026-11-22) · evidence: research/corpus.jsonl (2,793 works) · next lane: docs/product/CAPABILITY.md

## Diagnostic

1. **Who is this for?** Self-learners working through CS fundamentals
   (data structures, algorithms, networks, systems) who already use
   short-form video daily but can't retain what they watch. Not
   "students" in general — the wedge is one vertical with a crisp
   prerequisite graph and objectively checkable recall answers.
2. **What's the pain?** Watching educational shorts feels productive but
   doesn't stick: no retrieval, no spacing, no prerequisite ordering.
   Today's alternative is "watch, nod, forget" or heavyweight MOOCs with
   45-minute time blocks people won't start.
3. **Why now?** (a) Retrieval-practice and spaced-effect research is
   settled science with public implementations (Tachyon srs.rs, fsrs-rs)
   to build on; (b) OER supply (CS50, open textbooks, MIT OCW) is
   licensable seed content; (c) feed formats are habituated — the
   interaction cost of learning in this format is now zero.
4. **10-star version?** A daily feed that knows the topic graph you're
   climbing, quizzes you at the spacing interval your memory decays at,
   and only shows you what you're ready for — mastery levels, not
   follower counts.
5. **MVP?** Vertical feed of seeded CS-fundamentals clips (OER + licensed),
   one recall question per clip gating the next swipe, an FSRS-scheduled
   Review tab, and a visible prerequisite track. No creators, no follows,
   no comments in v0.
6. **Anti-goals (v0)?** No creator economy, no social graph, no
   infinite-scroll-without-recall (AGENTS.md invariant 8), no general
   "all academic topics" breadth — one vertical until the loop is proven.
7. **Working metric?** 7-day recall rate: percentage of recall questions
   answered correctly on *scheduled review* (not immediately). Secondary:
   D7 retention, review-tab completion rate. Watch time is explicitly
   NOT a metric (invariant 6).

## Why the wedge is CS fundamentals

| Criterion | CS fundamentals | Math | Exam tracks (AP/IB) |
|---|---|---|---|
| Prerequisite graph crispness | high (DAG is canonical) | high but broad | per-curriculum |
| Objectively checkable recall | high (predict output, pick invariant) | high | high but prose-heavy |
| Audience self-serve motivation | high (career-driven adults) | medium | high but seasonal |
| OER seed supply | excellent (CS50, OCW, open textbooks) | good | fragmented |
| Estate adjacency | dogfoods graphkit patterns daily | low | low |

## Mechanic → evidence map (invariant 7 anchors)

| Mechanic | Anchor works (in corpus) | Claim shipped against |
|---|---|---|
| Post-video recall gate | Karpicke & Roediger 2008 "The Critical Importance of Retrieval for Learning"; retrieval-practice query slice | retrieval > restudy for retention |
| FSRS Review tab | distributed-practice slice incl. Cepeda et al 2006 quantitative review | spacing beats massing |
| One concept per clip + length caps | Mayer 2003 "Nine Ways to Reduce Cognitive Load in Multimedia Learning" | cognitive-load discipline raises transfer |
| Prerequisite track gating | knowledge-prerequisite query slice | ordered prerequisite structure aids learning |
| Mastery framing, no streaks | self-determination query slice (Ryan & Deci lineage) | competence+autonomy sustain motivation; guilt mechanics harm it |
| Challenge mode (predict next step) | desirable-difficulties query slice (Bjork lineage) | generation/prediction improves retention |

Full per-query provenance: `research/SEARCH-LOG.md`. Curation pass
(pending) will pin exact work IDs per mechanic.

## Risks

- **Recall-question quality is the product.** Bad questions poison the
  loop. Mitigation: questions seeded hand-written per clip; rubric in
  content README; never auto-generated in v0.
- **Feed-shaped = binge-shaped.** The format pulls toward entertainment
  use. Mitigation: recall gate is structural, not optional; session view
  is capped by scheduled-review count, not scroll depth.
- **Licensing of seed clips.** OER licenses vary (CC-BY vs CC-BY-NC vs
  terms-of-use). Mitigation: content/ manifest records license + source
  per clip; NC and unknown licenses excluded from v0.

## Recommendation

**GO** — build the MVP loop for CS fundamentals. The thesis is falsifiable
cheaply: if 7-day recall on scheduled reviews doesn't beat a watch-only
control group in a 40-user cohort, the format doesn't carry learning and
the project stops or pivots to pure SRS without feed.
