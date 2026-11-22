#!/usr/bin/env python3
"""Harvest the edureel literature corpus from OpenAlex.

Exports works matching focused education-theory and operant-conditioning
queries to research/corpus.jsonl (one JSON object per line, abstracts
rebuilt from the inverted index) and writes research/SEARCH-LOG.md with a
reproducible record of every query pass.

Reproducible: run `python3 scripts/harvest_openalex.py` any time; queries
and filters are versioned here. OPENALEX_API_KEY from env raises the rate
budget; without it we join the polite pool via a mailto parameter.
"""

from __future__ import annotations

import json
import os
import sys
import time
import urllib.parse
import urllib.request
from datetime import date, datetime, timezone
from pathlib import Path

BASE = "https://api.openalex.org"
REPO = Path(__file__).resolve().parent.parent
OUT_JSONL = REPO / "research" / "corpus.jsonl"
OUT_LOG = REPO / "research" / "SEARCH-LOG.md"
MAILTO = "wyatt@wyatt.au"

SELECT = ",".join(
    [
        "id",
        "doi",
        "display_name",
        "publication_year",
        "type",
        "authorships",
        "primary_location",
        "open_access",
        "cited_by_count",
        "primary_topic",
        "abstract_inverted_index",
    ]
)

# One concept per query (OpenAlex search ranks relevance; broad queries
# surface keyword-coincidences). Pages per query tuned so the union of
# unique works comfortably exceeds the 1000-work corpus target.
QUERIES: list[dict[str, str]] = [
    {"topic": "operant conditioning", "search": "operant conditioning", "pages": "2"},
    {"topic": "reinforcement schedules", "search": "reinforcement schedule behavior", "pages": "2"},
    {"topic": "retrieval practice", "search": "retrieval practice testing effect", "pages": "2"},
    {"topic": "spaced repetition", "search": "spaced repetition distributed practice", "pages": "2"},
    {"topic": "microlearning", "search": "microlearning", "pages": "2"},
    {"topic": "gamification", "search": "gamification education motivation", "pages": "2"},
    {"topic": "educational video", "search": "educational video learning outcomes", "pages": "2"},
    {"topic": "short-form video", "search": "short-form video learning social media", "pages": "2"},
    {"topic": "cognitive load", "search": "cognitive load multimedia learning", "pages": "2"},
    {"topic": "mastery learning", "search": "mastery learning", "pages": "2"},
    {"topic": "prerequisite structure", "search": "knowledge prerequisite structure learning", "pages": "2"},
    {"topic": "behaviorism in class", "search": "behaviorism classroom reinforcement education", "pages": "2"},
    {"topic": "motivation (SDT)", "search": "self-determination theory education", "pages": "2"},
    {"topic": "desirable difficulties", "search": "desirable difficulties learning", "pages": "2"},
]


def rebuild_abstract(inverted: dict | None) -> str:
    if not inverted:
        return ""
    positions: dict[int, str] = {}
    for word, idxs in inverted.items():
        for i in idxs:
            positions[i] = word
    return " ".join(positions[i] for i in sorted(positions))


def fetch(url: str) -> dict:
    key = os.environ.get("OPENALEX_API_KEY")
    last_err: Exception | None = None
    for attempt in range(4):
        req = urllib.request.Request(url)
        req.add_header("User-Agent", f"edureel-harvest/0.1 (mailto:{MAILTO})")
        if key:
            req.add_header("Authorization", f"Bearer {key}")
        try:
            with urllib.request.urlopen(req, timeout=60) as resp:
                if resp.status != 200:
                    raise RuntimeError(f"HTTP {resp.status} for {url}")
                return json.loads(resp.read().decode())
        except Exception as exc:  # noqa: BLE001 - backoff ladder, then surface URL
            last_err = exc
            time.sleep((2, 12, 25)[min(attempt, 2)])  # 401s here are burst limits; wait them out
    raise RuntimeError(f"fetch failed after retry: {url}") from last_err


def compact(work: dict) -> dict:
    venue = ((work.get("primary_location") or {}).get("source") or {}).get("display_name")
    authors = [
        a.get("author", {}).get("display_name")
        for a in work.get("authorships", [])[:6]
        if a.get("author")
    ]
    return {
        "id": work["id"],
        "doi": work.get("doi"),
        "title": work.get("display_name"),
        "year": work.get("publication_year"),
        "type": work.get("type"),
        "authors": authors,
        "venue": venue,
        "oa_url": (work.get("open_access") or {}).get("oa_url"),
        "cited_by_count": work.get("cited_by_count"),
        "topic": (work.get("primary_topic") or {}).get("display_name"),
        "queries": [],
        "abstract": rebuild_abstract(work.get("abstract_inverted_index")),
    }


def main() -> int:
    corpus: dict[str, dict] = {}
    log_rows: list[tuple[str, str, int, str]] = []
    started = datetime.now(timezone.utc).isoformat(timespec="seconds")

    for q in QUERIES:
        pages = int(q["pages"])
        total = None
        cursor = "*"
        fetched = 0
        for _ in range(pages):
            params = urllib.parse.urlencode(
                {
                    "search": q["search"],
                    "select": SELECT,
                    "per-page": 100,
                    "cursor": cursor,
                    "mailto": MAILTO,
                }
            )
            data = fetch(f"{BASE}/works?{params}")
            total = data["meta"]["count"]
            for work in data.get("results", []):
                fetched += 1
                entry = corpus.setdefault(work["id"], compact(work))
                entry["queries"].append(q["topic"])
            cursor = data["meta"].get("next_cursor")
            if not cursor:
                break
            time.sleep(1.2)  # polite pool pacing; 401s are burst limits, not auth
        log_rows.append((q["search"], q["topic"], total or 0, f"{fetched} exported"))
        print(f"{q['search']!r}: total={total} exported={fetched} unique_so_far={len(corpus)}", file=sys.stderr)

    OUT_JSONL.parent.mkdir(parents=True, exist_ok=True)
    with OUT_JSONL.open("w", encoding="utf-8") as fh:
        for entry in corpus.values():
            fh.write(json.dumps(entry, ensure_ascii=False) + "\n")

    today = date.today().isoformat()
    lines = [
        "# Literature search log — edureel corpus",
        "",
        f"Corpus: {len(corpus)} unique works | exported: {OUT_JSONL.relative_to(REPO)}",
        f"Last run: {started} | exported works carry the `queries` that matched them.",
        "Filters: none beyond defaults (per-query page caps in `scripts/harvest_openalex.py`).",
        "",
        "| Database | Date searched | Query | Focus | Total matches | Exported |",
        "| --- | --- | --- | --- | ---: | ---: |",
    ]
    for search, focus, total, exported in log_rows:
        lines.append(f"| OpenAlex | {today} | `search={search}` | {focus} | {total:,} | {exported} |")
    lines += ["", "To reproduce: `python3 scripts/harvest_openalex.py` (idempotent; rewrites corpus)."]
    OUT_LOG.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"corpus={len(corpus)} -> {OUT_JSONL}", file=sys.stderr)
    return 0 if len(corpus) >= 1000 else 1


if __name__ == "__main__":
    raise SystemExit(main())
