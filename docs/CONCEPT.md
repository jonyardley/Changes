# Changes — a hands-free jazz ear trainer (working title)

*A functional-first jazz ear training app for iOS with a Crux (Rust) core, designed around commute-length, audio-first sessions. Concept v0.1, 2026-07-07.*

## The one-liner

**Sonofield's hands-free UX × Toned Ear's progression training × Coker's jazz progression cells × real spaced repetition** — the app that trains a jazz pianist to hear changes, on the train, with just headphones.

## Why this app (from the research)

- Pedagogy consensus: functional/contextual hearing + progression *chunks* + singing transfer to real playing; isolated interval drilling doesn't.
- Market: nobody offers jazz harmony training that's hands-free, offline, and SRS-driven. Nearest neighbors each hold one piece.
- The commute constraint isn't a limitation — short daily sessions are *pedagogically optimal* (ears fatigue; 4×5 min beats 1×20 min).

## Design principles

1. **Audio-first, screen-optional.** Every exercise must be completable with headphones only. The screen is an enhancement, never a requirement.
2. **Functional over intervallic.** Everything is heard against a key center (cadence or drone establishes it). No "name this interval" out of context.
3. **Decomposition over flashcards.** Chord exercises teach the jazz listening method: bass → 3rd → 7th → colors.
4. **Chunks linked to tunes.** Every progression cell is anchored to named standards ("this is the 'Lady Bird' turnaround").
5. **Error-driven (Banacos pattern).** A miss triggers comparison — replay cadence ↔ missed note, or A/B the confused chord qualities — never just "wrong, next."
6. **Honest gamification.** Streak = minutes of real work; rewards weight hard items and production (singing); SRS resurfaces weak items on decay curves.
7. **Jazz texture.** Rootless voicings, shells, swing feel, ride patterns — not root-position block piano.

## Core modes / user journeys

### 1. Pocket Session (the flagship — hands-free commute mode)

Phone in pocket, headphones in. An auto-paced audio loop:

1. Session opens with today's SRS queue + current-rung material (10/15/20 min, user-set).
2. Each item: **context** (cadence or drone in a random key) → **question audio** (a note, chord, guide-tone line, or 2–8 bar progression in combo texture) → **thinking gap** (configurable) → **spoken answer** ("that was… flat nine on a G7" / "minor two-five-one in C minor").
3. Self-report via minimal input: earbud tap / raise-to-speak "got it" vs "missed it", or fully passive "shadow mode" (no grading, pure exposure — for the crowded-train days).
4. On a miss: the Banacos loop — alternate the answer with the context, or A/B the two qualities you confuse, before moving on.
5. Session ends with a 30-second recap: what's due tomorrow, what rung progress was made.

### 2. Call & Response (voice input — walking, not driving)

Mic-graded singing, the highest-transfer activity that needs no instrument:

- "Sing Do" after hearing a tune snippet (tonic orientation).
- "Sing the 3rd / the 7th / the #9" of a played chord.
- Sing back a 2-bar phrase; sing the guide-tone line through a ii-V-I.
- Pitch detection grades against expected scale degrees; tolerance-based scoring (octave-agnostic).

### 3. At the Piano (evening companion mode)

The commute trains recognition; this closes the loop to production:

- Play-what-you-hear: app plays a phrase/voicing, you play it back; mic (chroma/onset detection) or MIDI-over-USB/Bluetooth grades it.
- Voicing identification and reproduction (rootless A/B, quartal, upper structures).
- "Touch real music" session-closer: a tune from your list, play along with generated changes.

### 4. Tune Workbench (the transcription bridge)

Productizes the iReal-Pro-blind-transcription hack:

- Pick a standard → hear its changes rendered in combo texture, *chart hidden*.
- Identify the form, then landmark chords, then cell-by-cell — graded progressively.
- Ladder of texture: clean piano → jazz trio voicings → (later) real-recording excerpts.
- Ties into Pocket Session: cells you missed become SRS items.

## Curriculum (maps to the research ladder)

| Level | Content | Exercise types |
|---|---|---|
| 0 | Tonic orientation | Find/sing Do; resolve note to Do |
| 1 | Diatonic degrees (major) | Cadence → degree ID; sing requested degree |
| 2 | Minor + chromatic degrees | All 12 colors; contextual intervals |
| 3 | Melodic phrases | Sing-back; degree dictation of fragments |
| 4 | Chord qualities | Decomposition drills: bass/3rd/7th; maj7 m7 7 m7b5 dim7; inversions |
| 5 | Guide tones & ii-V-I | 3rd/7th line tracking; major/minor ii-V-I; resolved vs deceptive V |
| 6 | Bass lines & cells | Root tracking; turnarounds, blues variants, rhythm changes, back-door, tritone sub — each linked to standards |
| 7 | Colors & voicings | Extensions/alterations as colors; upper structures; voicing types |
| 8 | Whole tunes | Tune Workbench mastery; form + landmarks in 1–2 listens |

Progression is gated by SRS mastery, not XP. Users can place-test past early rungs.

## Architecture (Crux)

```
┌─────────────────────────────────────────────┐
│ Swift shell (SwiftUI)                       │
│  • Audio render: AVAudioEngine sampler      │
│    (SoundFont piano/bass/ride) from core-   │
│    emitted score events                     │
│  • Mic capture → buffers to core            │
│  • Speech synthesis (spoken answers),       │
│    earbud tap / remote-control events       │
│  • Background-audio session, lock-screen    │
│    controls (Pocket Session ≈ podcast app)  │
└──────────────▲───────────────▼──────────────┘
        effects (Capabilities)   events
┌─────────────────────────────────────────────┐
│ Rust core (crux_core)                       │
│  • Music theory engine: keys, chords,       │
│    voicings, progression-cell grammar,      │
│    voice-leading generator (guide tones)    │
│  • Exercise generator: rung + SRS queue →   │
│    concrete score (notes, timing, key)      │
│  • Session state machine (auto-paced        │
│    Pocket flow, Banacos error loop)         │
│  • SRS scheduler (FSRS-style decay model)   │
│  • Pitch detection (YIN/pYIN on mic         │
│    buffers) + sung-answer grading           │
│  • Progress store (via KV/storage effect)   │
└─────────────────────────────────────────────┘
```

- Core is pure and platform-free: exercise = deterministic function of (rung, SRS state, RNG seed) → fully testable in Rust, no simulator needed.
- Audio out as an effect: core emits `PlayScore(events)` — shell realizes it with a sampler. Core never touches an audio API.
- Pitch detection in Rust keeps grading logic testable and portable (Android/web shells later get it free).
- Everything offline; no accounts, no server. One-time purchase positioning.

## Alternate concepts considered

- **B. "Changes Gym"** — only Rung 5–6 (progression cells linked to standards), no curriculum below. Sharper wedge, but abandons the beginner ladder and the research says degree-hearing is the prerequisite. → Folded in as Levels 5–6 + Tune Workbench.
- **C. "Sing First"** — voice-input-first call-and-response app. Highest pedagogical transfer but hardest tech risk (pitch tracking UX) and awkward on a crowded train. → Folded in as the Call & Response mode, phase 2.

## Build phases

1. **MVP:** Core theory engine + Levels 0–2 + Pocket Session (hands-free loop, spoken answers, earbud-tap grading) + SRS. *This alone already beats Functional Ear Trainer + Sonofield for a jazz user.*
2. **Harmony:** Levels 4–5 (chord decomposition, guide tones, ii-V-I) with jazz voicing textures.
3. **Cells & tunes:** Level 6 + Tune Workbench (needs a small standards/changes dataset).
4. **Voice:** Call & Response with pitch detection; Level 3 sing-backs.
5. **Piano companion:** MIDI/mic play-what-you-hear; Level 7 voicings.
