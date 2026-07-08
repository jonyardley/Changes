# Answer input: commit → pick → feedback (replaces self-grade)

*Tier 3 spec (schema migration = sensitivity override). Rides with the
first implementation PR. Agreed with Jon 2026-07-08 (brainstorm session);
supersedes the reveal-then-self-grade interaction from the design handoff
and DESIGN_BRIEF §"In session" point 3.*

## Problem

Self-grading (Got it / Missed it) gives the SRS subjective data and
invites the "illusion of mastery" RESEARCH.md warns about. The user should
provide the answer, get graded feedback, and accrue objective progress —
without weakening the recall discipline the Benbassat method depends on.
The instrument (MIDI, Phase 4) remains the production answer channel; this
is the recognition channel done honestly.

## Decisions

1. **Commit-then-pick** (option b of the brainstorm). The thinking gap
   stays visually clean — pure free recall. A deliberate tap ("I've named
   it") reveals the degree picker; the user taps the degree they already
   named; the reveal grades it. Rationale: showing options during the gap
   would turn recall into 12-way recognition (the dilution FET accepts and
   Changes exists to improve on) and would bend "sound before sign".
   Collapsing to picker-in-gap later is a one-seam change if dogfooding
   demands it.
2. **The picker shows the current rung's pool only** (7 degrees at rung 1,
   12 at rung 2), low → high, degree labels in the token vocabulary.
   Labels aren't notation; principle 7 holds.
3. **Feedback is honest and quiet.** Correct: confirmation + the
   resolution path plays (as today). Wrong: both degrees shown (yours and
   the answer), resolution plays, and continuing leads through the
   **Banacos compare loop seeded with the degree the user actually
   answered** — the static confusion-twin table becomes a fallback only.
   Compare on a miss stays mandatory (errors are the curriculum).
4. **Self-grade is removed entirely** — one honest path, no dual mode.
   "Not sure" is answered wrong by whatever was picked. The deferred
   pocket mode's earbud grading will need its own design when that bundle
   is picked up (taps can't pick from 12).
5. **SRS mapping unchanged in spirit**: correct → `Got` (FSRS Good),
   wrong → `Missed` (Again). The grade is now derived, not self-reported.
6. **Review logs record the answer**: `answered` column (semitones 0–11,
   INTEGER) joins `review_logs` — migration v2, the first real upgrade
   migration (fixture pattern in `MigrationTests` extends per the
   established convention). Old rows have `answered = NULL` (pre-input
   self-graded history remains valid).
7. **Straight to SwiftUI, no Claude Design round-trip** (Jon,
   2026-07-08): tokens and direction are settled; picker layout chosen
   from 2–3 SwiftUI snapshot variants in the follow-up PR.

## Flow (per item)

```text
context → question → gap ("name it" — clean screen)
  → tap: "I've named it"            [TapReady: Gap → Pick]
  → picker (rung pool, one tap)     [SubmitAnswer{degree}: Pick → Reveal]
  → reveal: verdict + answer + aural resolution
  → tap: continue                   [TapNext: Reveal → next item's context,
                                     or → Compare when wrong (mandatory)]
  → (compare A/Bs answered-vs-correct until ExitCompare)
```

Three taps per item (was two); the extra tap is the recall commitment and
fits the manual-pacing stance (decision 9).

## Core surface changes

- Events: `TapReveal`, `GradeGotIt`, `GradeMissedIt` → replaced by
  `TapReady`, `SubmitAnswer { degree }`, `TapNext`.
- New `Phase::Pick`; `ViewModel` gains `options: Vec<DegreeOption>`
  (label + semitones, precomputed in core) and `AnswerView` gains
  `verdict: Option<Verdict { your_label, correct }>`.
- `ReviewLog.answered: Option<Degree>`; compare side selection prefers
  the wrong answer over the twin table.

## Progress tracking unlocked (surfaces arrive with M5)

Objective per-skill accuracy; a real confusion matrix (the design's item
detail line "often confused with ♯4 — 5 of your last 6 misses" becomes
queryable from `review_logs(skill, answered)`); placement (M5) can grade
itself. No new storage beyond decision 6.

## Resolved

- **Picker layout** (2026-07-08): grid, chosen over strip and arc
  candidates rendered as snapshots at the rung 2 (12-option) worst case.
  Arc's labels overlapped illegibly at 12 options; grid stayed legible and
  kept generous touch targets at every rung size. Strip matched the
  duration-pills vocabulary but wrapped unevenly (a lone final pill) and
  buried the tonic marker among 11 similar pills. Implementation:
  `PickerGrid` in `ios/Changes/Views/Components/PickerGrid.swift`, wired
  into `RootView.pickContent`. The tonic cell carries the screen's one
  glow, tying the picker back to the resolution-path pedagogy shown at
  reveal.

## Open questions (non-blocking)

- Whether the reveal's continue tap doubles as "replay resolution" on a
  long-press — dogfooding call.
