# Roadmap

Single source of truth for what's being built and in what order. Update on
completion of each item; work should map to an item here (see CLAUDE.md →
Workflow → Always).

## Phase 0 — Foundations (current)

- [x] Pedagogy + market research (`docs/RESEARCH.md`)
- [x] Product concept (`docs/CONCEPT.md`)
- [x] Design brief for Claude Design (`docs/DESIGN_BRIEF.md`)
- [x] Design exploration: Pocket Session journeys, tokens, choreography
      prototype (`design/` — see `design/README.md`; direction 1a "Blue Hour
      Console")
- [x] MVP implementation plan (`docs/specs/mvp-plan.md` — milestones M0–M6)
- [ ] M0 — Scaffold: Crux workspace, xcodegen iOS project, justfile, CI,
      `Theme.swift` from the design token sheet, bindings walking skeleton
- [ ] M1 — Audio spike: `PlayScore` → AVAudioEngine sampler, background
      audio, Now Playing + remote commands. **Make-or-break; go/no-go gate
      before any feature work** (exit criteria in the plan)

## Phase 1 — MVP: Pocket Session, Levels 0–2

Functional degree training, hands-free. Beats Functional Ear Trainer +
Sonofield for a jazz user on its own. Build order and decisions:
`docs/specs/mvp-plan.md` (milestones M2–M6).

- [ ] Theory engine v1: keys, cadences, scale degrees (major/minor,
      chromatic)
- [ ] Exercise generator + session state machine (context → question →
      thinking gap → reveal; Banacos error loop)
- [ ] SRS scheduler (FSRS-style) + GRDB persistence (sync-ready schema)
- [ ] Pocket Session UI: pre-session, in-session states, lock screen /
      Dynamic Island, earbud-tap grading, aural reveal (resolution /
      decomposition playback — no TTS; lock-screen title shows the answer)
- [ ] Session recap + Ladder/progress hub (Levels 0–2 only)
- [ ] Onboarding + placement test (rungs 0–2)

## Phase 2 — Harmony: Levels 4–5

- [ ] Chord engine: qualities (maj7/m7/7/m7b5/dim7), inversions, jazz
      voicing textures (rootless, shells)
- [ ] Decomposition exercises (bass → 3rd → 7th)
- [ ] Guide-tone lines + ii-V-I recognition (major/minor; resolved vs
      deceptive dominants)
- [ ] Reveal-moment visual components: mini keyboard diagram (voicings) +
      guide-tone contour — post-answer only (sound before sign)

## Phase 3 — Cells & tunes: Level 6 + Tune Workbench

- [ ] Progression-cell grammar (turnarounds, blues variants, rhythm changes,
      back-door, tritone subs) linked to named standards
- [ ] Standards/changes dataset (small, curated)
- [ ] Tune Workbench: blind-transcription flow with self-revealing lead
      sheet; missed cells feed the SRS queue

## Phase 4 — Instrument I/O: MIDI call & response

*(Voice input/output dropped 2026-07 — no sung grading, no TTS. The
instrument is the production channel.)*

- [ ] CoreMIDI input (Bluetooth LE + USB), device pairing UX
- [ ] Played-answer grading in core (note-for-note, timing-tolerant)
- [ ] Call & response at the piano: phrases, "play the 3rd/7th/#9",
      Banacos comparison loop on misses
- [ ] Level 3 phrase-echo grading

## Phase 5 — Piano companion, expanded

- [ ] Acoustic-instrument note detection (mic buffers → core; onset +
      chroma), for pianos without MIDI
- [ ] Level 7: extensions/alterations as colors; voicing ID + reproduction

## Deliberately out of scope

Accounts, server/sync, subscriptions, social/leaderboards, Android/web
(the Crux core keeps the option open; not planned). **Voice interaction in
both directions** — no text-to-speech answers, no sung-answer grading
(dropped 2026-07; aural reveal + instrument I/O replace them).
