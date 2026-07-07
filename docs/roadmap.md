# Roadmap

Single source of truth for what's being built and in what order. Update on
completion of each item; work should map to an item here (see CLAUDE.md →
Workflow → Always).

## Phase 0 — Foundations (current)

- [x] Pedagogy + market research (`docs/RESEARCH.md`)
- [x] Product concept (`docs/CONCEPT.md`)
- [x] Design brief for Claude Design (`docs/DESIGN_BRIEF.md`)
- [ ] Design exploration: Pocket Session journey mockups (Claude Design)
- [ ] Scaffold Crux workspace: `jazzear-core`, `jazzear-ffi`, iOS app shell,
      justfile, CI (fmt/clippy/test)
- [ ] Spike: end-to-end audio loop — core emits `PlayScore` → shell renders
      cadence + note on AVAudioEngine sampler → background audio + lock-screen
      controls. **This is the make-or-break spike; do it before building
      features.**

## Phase 1 — MVP: Pocket Session, Levels 0–2

Functional degree training, hands-free. Beats Functional Ear Trainer +
Sonofield for a jazz user on its own.

- [ ] Theory engine v1: keys, cadences, scale degrees (major/minor,
      chromatic)
- [ ] Exercise generator + session state machine (context → question →
      thinking gap → reveal; Banacos error loop)
- [ ] SRS scheduler (FSRS-style) + GRDB persistence (sync-ready schema)
- [ ] Pocket Session UI: pre-session, in-session states, lock screen /
      Dynamic Island, earbud-tap grading, spoken answers
- [ ] Session recap + Ladder/progress hub (Levels 0–2 only)
- [ ] Onboarding + placement test (rungs 0–2)

## Phase 2 — Harmony: Levels 4–5

- [ ] Chord engine: qualities (maj7/m7/7/m7b5/dim7), inversions, jazz
      voicing textures (rootless, shells)
- [ ] Decomposition exercises (bass → 3rd → 7th)
- [ ] Guide-tone lines + ii-V-I recognition (major/minor; resolved vs
      deceptive dominants)

## Phase 3 — Cells & tunes: Level 6 + Tune Workbench

- [ ] Progression-cell grammar (turnarounds, blues variants, rhythm changes,
      back-door, tritone subs) linked to named standards
- [ ] Standards/changes dataset (small, curated)
- [ ] Tune Workbench: blind-transcription flow with self-revealing lead
      sheet; missed cells feed the SRS queue

## Phase 4 — Voice: Call & Response

- [ ] Mic capture → core pitch detection (YIN/pYIN)
- [ ] Sung-answer grading (octave-agnostic, tolerance-based)
- [ ] Level 3 sing-backs; "sing the 3rd/7th/#9" exercises

## Phase 5 — Piano companion

- [ ] MIDI (USB/Bluetooth) input; play-what-you-hear grading
- [ ] Level 7: extensions/alterations as colors; voicing ID + reproduction

## Deliberately out of scope

Accounts, server/sync, subscriptions, social/leaderboards, Android/web
(the Crux core keeps the option open; not planned).
