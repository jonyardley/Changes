# MVP Implementation Plan — Pocket Session (Phases 0–1)

*Tier 3 spec. Turns `design/` (Claude Design handoff, 2026-07) + `docs/CONCEPT.md`
into a build order. Last reviewed: 2026-07-07.*

## Problem

We have validated research, a concept, and a high-fidelity design with a
working choreography prototype. Nothing is built. The riskiest assumptions are
technical (hands-free audio loop on iOS), not product. Build in an order that
retires risk first and turns the design's state machine into the shippable
MVP: **Pocket Session, curriculum Levels 0–2, SRS, fully hands-free.**

## Design ↔ MVP reconciliation (read this first)

The design's hero content is Rung 4 (chord qualities, keyboard diagram at
reveal) because it shows the visual language best. **The MVP ships Rungs 0–2
(scale degrees) in the exact same session frame.** Nothing in the design
changes — the reveal simply shows a degree ("♭3 — resolves down to… do")
instead of a chord symbol, and the keyboard diagram / quality glyphs arrive
with Phase 2 content. Build the frame once; content types plug in.

## Key decisions

1. **Miss-comparison treatment: 2b two-cards** (not 2c pendulum). Maps 1:1 to
   the audio A/B alternation, simpler to build, identical hands-free
   behaviour. Pendulum stays on the canvas as a possible later delight pass.
   *(Open decision 1 from the design handoff — recommended, cheap to revisit.)*
2. **Tension palette: the system sheet (canvas 2a/4a) is canonical**, per the
   handoff's own recommendation. The prototype's coral-♭9 is a bug.
3. **Lock screen = Now Playing for MVP; Live Activity fast-follows.**
   `MPNowPlayingInfoCenter` + `MPRemoteCommandCenter` give transport, media
   keys, and earbud events — mandatory plumbing anyway. The mocked Live
   Activity/Dynamic Island layer adds phase text + progress; ship it in M6
   once the loop is proven.
4. **Earbud answer input is phase-contextual.** Headset taps arrive as remote
   commands (single = play/pause, double = next-track). The core interprets
   them by session phase: during reveal-awaiting-report, play/pause → `GotIt`,
   next-track → `MissedIt`; in compare, play/pause → `ExitCompare`; in all
   other phases, standard transport. Lock-screen *buttons* always keep podcast
   semantics (the design's "never remapped" rule applies to visible transport;
   contextual earbud taps are how ×1/×2 grading is physically possible).
5. **No voice, in either direction (stakeholder decision, 2026-07).** No
   text-to-speech answers, no sung-answer grading — the design handoff's
   spoken-answer references are superseded. The eyes-free reveal is **aural**:
   a degree resolves stepwise to the tonic (the resolution path names it —
   the Benbassat move); a chord replays as its decomposition arpeggio (bass →
   3rd → 7th → color). The Now Playing title updates at reveal ("It was: ♭3")
   for pocket-glance confirmation. If M2 dogfooding shows the aural reveal is
   ambiguous for some item types, the fallback option is short **pre-recorded
   human voice clips** (finite vocabulary, not TTS) — a stakeholder decision
   to take then, not built now.
6. **SRS: FSRS-style scheduler in `jazzear-core`.** Evaluate `fsrs-rs` (the
   Anki implementation) first — **check its license fits a paid closed-source
   app before adopting**; otherwise implement SM-2 behind the same trait so
   the scheduler can be swapped without touching callers.
7. **Fonts: bundle Space Grotesk + Newsreader** (both SIL OFL — bundling in a
   paid app is fine, keep the OFL notices). "Machine speaks grotesque, music
   speaks serif" is a token-level rule: `JazzearFont.ui*` vs
   `JazzearFont.music*`.
8. **Timings are config data, not constants in code.** The prototype's values
   (context 3.6s, question 2.6s, gap 2–8s default 4s, auto-continue 5.2s,
   compare alternation 1.7s) live in one core `SessionTiming` struct so
   tuning-by-ear doesn't mean hunting magic numbers.

## Milestones (each ≈ one PR-sized chunk; sequential unless noted)

### M0 — Scaffold
Workspace (`jazzear-core`, `jazzear-ffi`), xcodegen iOS project, justfile
(`just ios`, `ios-run`, `ios-gen`, `ios-logs` mirroring intrada), CI
(fmt/clippy/test + iOS build), `Theme.swift` with the full handoff token sheet
(`JazzearColor` incl. quality/tension/mastery ramps, `JazzearFont`,
`JazzearSpacing` 28px screen pad / 20–22 radius / 99 pill, glow styles),
bundled fonts. Bindings pipeline proven with a walking-skeleton
`Event::Ping → ViewModel`.

### M1 — Audio spike (make-or-break; go/no-go gate)
Core emits `PlayScore` (abstract score: notes/voicings, onsets, durations,
tempo) → shell realizes on `AVAudioEngine` + SoundFont sampler. Background
audio entitlement + `.playback` session; Now Playing info; remote commands
round-tripping as core events. **Exit criteria, on a real device:** phone
locked in pocket, headphones on → cadence → chord → (gap) → replay loop runs
for 5+ minutes with no glitches, dropouts, or drift; earbud tap lands as a
core event in every phase; interruption (Siri/call) pauses and resumes
correctly. If this fails, the product concept needs rework — nothing else
gets built first.

### M2 — Core session engine
Session state machine exactly per the design spec (`design/README.md`
"Interactions & Behavior"): `pre → [context → question → gap → reveal →
(compare)]* → recap`, pause/interrupted overlays, auto-continue off by
default, shadow mode. Exercise generator for Rungs 0–2 (cadence + degree
items, major → minor → chromatic), deterministic from (curriculum, SRS state,
seed). **Aural-reveal score generation** (resolution paths for degrees;
decomposition arpeggios for chords — decision 5). SRS scheduler + daily
queue. GRDB storage effects + migration v1 (`updated_at`/`deleted_at` from
day one). Effect-level tests assert the emitted `PlayScore`/timer sequences —
the choreography is tested without audio.

### M3 — Pocket Session UI
All canvas states, pixel-close to 1a/2b/4b: pre-session, context/listening
(level bars only), question ("?" pulse), gap (countdown ring), reveal
(degree/symbol on screen + aural reveal playback + Now Playing title update;
answer zones 25%↔100%), compare (2b two-cards), paused, interrupted (amber
banner), complete (ledger + "tonight at the piano"). Snapshot test +
VoiceOver + Dynamic Type per screen, plus the **screen-off pass**: every
interaction verified with display locked.

### M4 — Hands-free hardening
Route-change handling (headphones unplugged = pause, never speaker);
external-audio auto-hold with replay-item semantics ("no item is ever lost
to an interruption"); Now Playing answer-title polish; settings surface
(session length, gap 2–8s, auto-continue, shadow mode).

### M5 — Surrounding surfaces
Home hub (today card, week bars, "needs your ear"), Ladder (9 rungs, mastery
dots, 80% gating), item detail, session recap wiring, onboarding + placement
test (3a: welcome → context capture → mini-session placement → "you're here"
ladder moment).

### M6 — Ship prep
Live Activity + Dynamic Island layer (2e), app icon (♭ glyph tile),
TestFlight lane (fastlane match, mirroring intrada's setup), OFL notices,
one-time purchase wiring deferred until public TestFlight feedback.

## Testing strategy

- Core: state-machine transition tests; property checks on the generator
  ("every item's answer is derivable from its score"); seeded replay.
- Bridge: real-bridge round-trip test for every `Event`/`Effect`/`ViewModel`
  payload **before** it's wired to a screen (intrada #846 lesson).
- Storage: upgrade-path test per migration (populated-at-previous-version).
- UI: snapshot per state; manual device pass for audio feel each milestone.

## Risks & open questions

| Risk | Mitigation |
|---|---|
| Audio gaps/drift in background over long sessions | M1 gate on real device; sampler pre-scheduled, not per-note dispatched |
| Earbud remote-command behaviour varies by headphone vendor | M1 test matrix: AirPods, wired, generic BT; screen-tap always works as fallback |
| SoundFont quality (jazz piano feel) | Timbre is explicitly not the spec; pick a decent free SF2 now, revisit later |
| `fsrs-rs` license incompatible | SM-2 behind the same trait (decision 6) |
| Aural reveal ambiguous for some item types (can the user *tell* the answer from the resolution/decomposition alone?) | Dogfood in M2 with real commute use; per-type reveal design; fallback = pre-recorded human voice clips (decision 5 — stakeholder call, not TTS) |
| Live Activity update cadence limits | Deferred to M6; Now Playing carries MVP |

Open with stakeholder: none blocking. Decisions 1–4 above are recommendations
Jon can veto cheaply before M3 (visual) / M1 (input semantics) land.
