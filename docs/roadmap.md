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
- [x] M0 — Scaffold
  - [x] Rust workspace + quality rails (2026-07-07): `changes-core` crux
        walking skeleton (`Event::Ping → ViewModel`, bincode round-trip
        tests), `changes-ffi` placeholder, strict lints (unwrap denied),
        pinned toolchain, cargo-deny, CI — Rust-only so far
        (fmt/clippy/test/deny/gitleaks → required "CI OK"), justfile +
        pre-push hook, dependabot (cargo)
  - [x] xcodegen iOS project, `Theme.swift` from the design token sheet,
        bundled fonts (Space Grotesk + Newsreader, OFL notices alongside)
        (2026-07-07)
  - [x] Bindings pipeline (uniffi =0.29.4 + cargo-swift 0.9.0 +
        facet typegen): `Event::Ping → ViewModel` through a real bridge
        round-trip (`crates/changes-ffi/src/ffi.rs` test + on-sim launch);
        `just ios / ios-run / ios-gen / ios-build / ios-logs` live
        (2026-07-07)
  - [x] iOS CI: macos-26 runner job regenerating bindings from the PR's
        core (content-addressed cache — staleness impossible by
        construction, no committed bindings to diff) + building the app,
        added to CI OK's `needs` (2026-07-07)
- [x] CD — TestFlight lane (2026-07-08, mvp-plan M6 item pulled forward):
      fastlane match + `release-testflight.yml` (v* tag / manual dispatch,
      signed release-core .ipa → TestFlight, never per-PR); CI hardening in
      the same pass (path-filtered macOS job with a strict CI OK check,
      DerivedData restore/save, per-merge launch screenshot artifact,
      dependabot uniffi lockstep ignore). One-time signing bootstrap +
      secrets: `docs/RELEASING.md` — **the lane is dormant until Jon adds
      the six secrets**.
- [ ] M1 — Audio spike: `PlayScore` → AVAudioEngine sampler; glitch-free,
      sample-accurate tap-driven playback; interruption + route-change
      handling. **Make-or-break; go/no-go gate before any feature work**
      (exit criteria in the plan)
  - [x] Pipeline code (2026-07-08): `PlayScore` effect + spike session
        machine in core (tap-paced context → question → reveal over E♭
        chords, effect-level tests); shell `ScorePlayer`
        (AVAudioSequencer-pre-scheduled sampler + GeneralUser GS SoundFont),
        `.playback` session, interruption + route-change → pause events;
        verified on-sim (`--spike-autotap` soak harness, no errors)
  - [ ] **The gate itself**: Jon runs the exit criteria on a real device —
        headphones, 5+ min back-to-back tap-driven playback, no
        glitches/drift; Siri/call interruption and headphones-unplugged
        pause correctly. *2026-07-08: Jon reviewed the spike on the
        simulator and chose to press on with core work in parallel.
        Decision (2026-07-08): parked until the first TestFlight build —
        run the exit criteria then, before M3 UI investment.*

## Phase 1 — MVP: Pocket Session, Levels 0–2

Functional degree training — foreground, touch-driven, user-paced (every
step a tap: start, reveal, grade). Beats Functional Ear Trainer +
Sonofield for a jazz user on its own. Build order and decisions:
`docs/specs/mvp-plan.md` (milestones M2–M6).

- [x] Theory engine v1 (2026-07-08): pitch classes, major/minor keys, the
      12 chromatic degree colors with per-mode labels, transposable
      ii–V–I / iiø–V–i cadence voicings, and Benbassat resolution paths
      (shortest-way-home direction rule, one tunable seam —
      `crates/changes-core/src/theory/`)
- [x] Exercise generator + session state machine (2026-07-08): seeded
      deterministic generator (rungs 0–2 pools, one key per session, no
      back-to-back repeats), `pre → [listening → gap → reveal →
      (compare)]* → recap` machine — manual pacing, Banacos A/B compare
      loop with fixed confusion twins, aural reveal = resolution-path
      playback, pause/resume replays the phase. Replaced the M1 spike app.
- [ ] SRS scheduler (FSRS-style) + GRDB persistence (sync-ready schema)
  - [x] Core half (2026-07-08, spec `docs/specs/srs-persistence.md`):
        `fsrs` crate (BSD-3, burn-free ≥6.3) behind a first-party
        `Scheduler` trait; skills = (mode, degree); queue = due → unseen →
        upcoming, single-mode sessions; grades feed the scheduler and emit
        typed `Storage` effects (load-before-audio, fire-and-forget saves,
        failures surfaced); deterministic ulid review logs. Shell fulfils
        storage with an in-memory store for now.
  - [x] Shell half (2026-07-08): `GrdbReviewStore` (GRDB 7.11, synchronous
        by recorded decision, upserts + tombstone-filtered reads, degraded
        in-memory fallback surfaced in UI), migration v1 per the spec
        schema, `ChangesTests` target (store round-trip + skill-key codec +
        populated-at-v1 upgrade test) run by `just ios-test` and CI.
- [x] Pocket Session UI (2026-07-08): all canvas states — pre-session
      (190px play, duration pills wired to session length), context (level
      bars), question ("?" pulse), gap, reveal (answer zones 25%↔100%),
      compare 2b two-cards with playing-side glow, user-paused card vs
      amber interrupted banner (new core `PauseState`), recap ledger +
      "tonight at the piano" card. Context/question split into separate
      phases + scores. 11 snapshot tests (pinned iPhone 16 / iOS 26.5,
      optimised PNGs) + VoiceOver labels + Dynamic Type variant.
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
