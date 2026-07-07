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
- [ ] M0 — Scaffold
  - [x] Rust workspace + quality rails (2026-07-07): `changes-core` crux
        walking skeleton (`Event::Ping → ViewModel`, bincode round-trip
        tests), `changes-ffi` placeholder, strict lints (unwrap denied),
        pinned toolchain, cargo-deny, CI — Rust-only so far
        (fmt/clippy/test/deny/gitleaks → required "CI OK"), justfile +
        pre-push hook, dependabot (cargo)
  - [ ] xcodegen iOS project, `Theme.swift` from the design token sheet,
        bundled fonts
  - [ ] Bindings pipeline (uniffi + typegen): `Event::Ping → ViewModel`
        through a real bridge round-trip
  - [ ] iOS CI: macOS-runner job building the iOS app + generated-bindings
        freshness check (regen, fail on diff), added to CI OK's `needs`
        (completes mvp-plan M0's "CI (fmt/clippy/test + iOS build)")
- [ ] M1 — Audio spike: `PlayScore` → AVAudioEngine sampler; glitch-free,
      sample-accurate tap-driven playback; interruption + route-change
      handling. **Make-or-break; go/no-go gate before any feature work**
      (exit criteria in the plan)

## Phase 1 — MVP: Pocket Session, Levels 0–2

Functional degree training — foreground, touch-driven, user-paced (every
step a tap: start, reveal, grade). Beats Functional Ear Trainer +
Sonofield for a jazz user on its own. Build order and decisions:
`docs/specs/mvp-plan.md` (milestones M2–M6).

- [ ] Theory engine v1: keys, cadences, scale degrees (major/minor,
      chromatic)
- [ ] Exercise generator + session state machine (context → question →
      thinking gap → reveal; Banacos error loop)
- [ ] SRS scheduler (FSRS-style) + GRDB persistence (sync-ready schema)
- [ ] Pocket Session UI: pre-session, in-session states, on-screen touch
      pacing (tap to start, tap to reveal, tap to grade — the grade tap
      advances; no timers, no auto-continue), aural + on-screen reveal
      (resolution / decomposition playback — no TTS)
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
both directions** — no text-to-speech answers, no sung-answer grading, no
voice commands or speech recognition (dropped 2026-07; aural reveal +
instrument I/O replace them). Audio is output-only music; every input is a
touch on the screen (earbud-tap control is part of the deferred pocket
mode below).

**Deferred, not rejected** (2026-07): the auto-paced + hands-free bundle
("pocket mode") — timed reveal / auto-continue, passive "shadow mode", and
the pocket layer (background audio, lock screen / Now Playing / Live
Activity, earbud-tap control). One post-MVP exploration, revisited after
dogfooding the manual loop. The MVP is foreground, screen in hand; every
step is a deliberate tap (start, reveal, grade).
