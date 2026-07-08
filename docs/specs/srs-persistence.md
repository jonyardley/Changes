# SRS scheduler + persistence (M2b)

*Tier 3 spec (storage schema = sensitivity override). Rides with the first
implementation PR. Last reviewed: 2026-07-08.*

## Problem

M2a's sessions are seeded-random. The product promise is that a spaced-
repetition engine decides what the user practices each day, and that
practice history survives forever on-device (the phone is the only copy).
Two halves: the scheduler (core, this PR) and GRDB persistence (shell, the
follow-up PR); the storage schema is fixed here so both halves agree.

## Decisions

1. **Skill identity = (mode, degree)** — 24 colors. Hearing ♭3 against a
   minor key is one skill regardless of which key the session picked;
   sessions keep choosing keys freely, so a skill is exercised across keys
   by construction (transfer, not memorising one key's timbre).
2. **Algorithm: the `fsrs` crate (≥6.3), BSD-3-Clause, behind a
   first-party `Scheduler` trait.** Evaluated per mvp-plan decision 6
   (2026-07-08): burn/ML deps removed in 6.3.0, ~40 KB code on an
   aarch64-apple-ios static lib, all-permissive transitive tree (passes
   deny.toml), canonical implementation maintained by the algorithm's
   authors, ships in paid closed-source AnkiMobile. The trait keeps SM-2
   (or first-partying the ~250 lines of FSRS-6 equations) a drop-in swap
   and keeps `fsrs` types off the FFI bridge — bridge/storage carry only
   our own fields.
3. **Grade mapping**: the UI's honest two-button self-grade maps
   `MissedIt → Again`, `GotIt → Good`. Hard/Easy arrive only if the UI
   ever earns more buttons — the schema stores our `Grade`, not FSRS's.
4. **Time arrives via events** (core determinism): `StartSession` carries
   `now_ms` (epoch millis) alongside `seed`; one session uses one `now`.
   FSRS operates in whole elapsed days computed from `last_reviewed_at`.
5. **Desired retention 0.9** (FSRS default) — a constant in core config,
   not persisted, tunable later without migration.
6. **Session queue**: due skills first (most overdue first), then unseen
   skills in curriculum order from the current rung's pool, then (if the
   session still has room) the soonest-due remainder. Rung gating (80%)
   lands with the Ladder (M5); until then the rung is the existing const.
7. **Ulid exception**: `review_logs` rows get client-minted ulids
   (canonical id rule); `review_states` uses the natural key `skill`
   (mode:semitones, e.g. `major:3`) — one row per skill by definition, and
   a surrogate id would only invite duplicates. Both tables carry
   `updated_at` + soft-delete `deleted_at` from day one (sync-ready).

## Schema (fixed here; GRDB migration v1 implements it verbatim)

```sql
CREATE TABLE review_states (
  skill            TEXT PRIMARY KEY,  -- "major:3" / "minor:10"
  stability        REAL NOT NULL,
  difficulty       REAL NOT NULL,
  last_reviewed_at INTEGER NOT NULL,  -- epoch ms
  due_at           INTEGER NOT NULL,  -- epoch ms
  updated_at       INTEGER NOT NULL,
  deleted_at       INTEGER            -- soft delete, NULL = live
);

CREATE TABLE review_logs (
  id          TEXT PRIMARY KEY,       -- client-minted ulid
  skill       TEXT NOT NULL,
  grade       TEXT NOT NULL,          -- "got" | "missed"
  reviewed_at INTEGER NOT NULL,       -- epoch ms
  updated_at  INTEGER NOT NULL,
  deleted_at  INTEGER
);
CREATE INDEX idx_review_logs_skill ON review_logs(skill, reviewed_at);
```

Migrations append-only/forward-only; every migration ships with a
populated-at-previous-version upgrade test (CLAUDE.md invariant).

## Effects & choreography

```text
StartSession{seed, now_ms}
  → Storage(LoadReviews) ──Loaded{states}──► build queue → generate items
                                             → Listening (as M2a)
Grade tap (got/missed)
  → scheduler.review(state, grade, now) → model updates immediately
  → Storage(SaveReview{state, log})  — fire-and-forget for UI flow;
    a Failed output surfaces in ViewModel.error (never a silent success)
  → advance (unchanged M2a flow)
```

- `StorageOperation { LoadReviews, SaveReview { state, log } }`,
  `StorageOutput { Reviews(Vec<ReviewStateRecord>), Ack, Failed { message } }`.
- Loading failure degrades gracefully: the session still runs (all-new
  queue) with the error surfaced — practice is never blocked by a broken DB.
- The shell (next PR) fulfils Storage with GRDB off the main actor,
  mirroring intrada's `ItemStore` protocol / `LibraryStore` split.

## Open questions (non-blocking)

- Same-day re-reviews: FSRS is day-granular; a skill missed and re-seen in
  one session re-schedules from the same-day state (elapsed 0) — accepted,
  Anki does the same.
- Per-user parameter optimisation (the crate ships the optimizer at no
  extra dependency cost) — post-MVP, needs review-log volume first.
