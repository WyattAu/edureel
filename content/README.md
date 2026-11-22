# content/ — seed data (the only content surface; the app never hardcodes clips)

Files:
- `topics.json` — topic prerequisite graph (consumed by crates/topics + web track view)
- `clips.json` — clip units. v0 clip kind is `card`: an original micro-lesson
  (~60s read) authored in this repo, license CC-BY-4.0 (ours). Video-kind clips
  (`kind: "video"`) exist in the schema for the post-MVP curation pass but are
  ABSENT from seeds until their license is verified — CS50/MIT OCW/Khan are
  CC BY-NC-SA and therefore excluded per docs/product/CAPABILITY.md constraints.
- `questions.json` — exactly one hand-authored recall question per clip
  (invariant: never auto-generated), with the correct option and a
  misconception note explaining the trap.

Rules for curators:
1. Every clip entry must carry `license` and `source`; unknown ⇒ do not add.
2. One question per clip; the question must test recall of the clip's core
   claim, not trivia.
3. Topics may exist without clips (feed shows "unlocked, content pending");
   clips must never reference a topic absent from topics.json.
4. Coverage in v0: the DAG's entry layer (6 topics) is fully seeded; the rest
   of the graph is structural and grows topologically as cards are written.
