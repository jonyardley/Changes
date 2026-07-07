# Design Brief — "Changes" (working title)

*A hands-free jazz ear training app for iOS. Brief for design exploration / journey mockups, v0.1 — 2026-07-07.*

## What we're building

An iOS app that trains a jazz pianist's ear — hearing notes against a key, jazz chord qualities, and chord progressions — designed for **commute-length sessions with headphones, phone in pocket**. The pedagogy is "functional-first": every sound is heard against a key center, mistakes trigger comparison-replay rather than "wrong, next," and a spaced-repetition engine decides what you practice each day.

**The one-liner:** the podcast-app-simple ear trainer that teaches you to hear jazz changes on the train.

**Positioning:** no app today combines jazz harmony + hands-free audio-first UX + spaced repetition. Nearest references: Sonofield's "Pocket Mode" (right UX, no harmony), EarMaster (right content, dated tap-heavy UX), Toned Ear (right exercise, wrong texture). We are the intersection.

## The user

- Adult amateur/intermediate jazz pianist (our archetype: a software professional who plays jazz piano, practices evenings, commutes ~30–60 min daily by train).
- Musically literate — knows what a ii-V-I and a #9 are; do **not** dumb down terminology.
- Time-poor; the commute is found time. Evening piano time is precious and shouldn't be spent tapping quiz buttons.
- Allergic to Duolingo-style gamification theatre; motivated by genuine skill progress ("I picked up a tune in two listens last week").

## Design principles (non-negotiable)

1. **Audio-first, screen-optional.** Every core exercise must be completable with the phone in a pocket. Screens support and enrich, never gate. Think "podcast app with a brain," not "quiz app with audio."
2. **One thumb, glanceable.** When the screen *is* used (crowded train, standing), everything reachable one-handed; state understandable in a half-second glance.
3. **Calm confidence over gamification.** No lives, leagues, XP explosions, or streak-panic. Progress = mastery and minutes of real work. Celebrations are understated and jazz-cool, not confetti.
4. **Sound has visual character.** Keys, chord qualities, and tension colors (b9, #13…) are the content — give them a consistent, learnable visual language (color/shape) that reinforces the ear learning rather than decorating it.
5. **Errors are the curriculum.** A miss opens a comparison moment (replay/alternate the two confused sounds). Design this as the most cared-for interaction in the app — it should feel like a patient teacher, not a penalty.
6. **Dark-mode native.** Primary contexts are trains, evening pianos, pockets. Design dark-first; light mode secondary.
7. **Sound before sign.** Visualizations of musical content (keyboard diagrams, guide-tone contours, lead-sheet bars) appear only at the **answer reveal** and in review/progress screens — never while the user is listening or answering. The ear works first; the eye confirms. No engraved staff notation anywhere — chord symbols, mini keyboard diagrams, and simple voice-leading contour lines are the vocabulary.

**Feel/mood:** late-night jazz club meets modern instrument. Warm darks, one or two rich accent hues, generous type, tactile controls. References to riff on: Teenage Engineering restraint, Moleskine/real-book texture cues, the calm of Apple Podcasts/Overcast, the focus of Endel. Avoid: cartoon mascots, arcade UI, music-school clip-art (treble clefs everywhere).

## Journeys to mock (in priority order)

### Journey 1 — Pocket Session (the flagship; design this first)

Daily hands-free training loop. The user experience is mostly *auditory*; the design task is the thin visual shell around it plus the audio-flow choreography.

1. **Start:** open app → today's session is front and center ("18 min · Level 4: Chord qualities · 12 reviews due"). One giant play button. Optional: duration override (10/15/20), "shadow mode" toggle (listen-only, no grading — for zero-attention days).
2. **In session (screen locked / in pocket):** lock-screen & Dynamic Island presence like a podcast: exercise counter, elapsed/remaining, pause. Earbud/tap gestures = "got it" / "missed it." The answer arrives **aurally, never as synthesized speech** (2026-07 decision — no TTS): a degree resolves stepwise to the tonic, a chord replays as its decomposition arpeggio (bass → 3rd → 7th → color), and the lock-screen title updates ("It was: ♭3") for a pocket glance.
3. **In session (screen open):** a calm "now playing" canvas: current key, what's being asked (e.g., "Quality of this chord?"), a subtle countdown of the thinking gap, then the revealed answer ("G7♭9") with its visual identity. Big tap zones: **Got it / Missed it**.
4. **Miss flow (the Banacos loop):** the app alternates the missed sound with its context or its confusable twin ("m7♭5 vs dim7 — hear the 7th move"). Visualize the comparison simply (two cards A/B-ing). User exits the loop when ready.
5. **Session end:** 30-second recap — items mastered, weakest confusion pair, what's due tomorrow, rung progress. One line of genuine encouragement, then a "tonight at the piano" nudge (ties to Journey 3).

Key states to cover: pre-session, playing, thinking-gap, answer reveal, miss/comparison, paused, interrupted (train announcement → auto-pause on route audio?), session complete.

### Journey 2 — Onboarding & placement

1. Welcome: the promise in one screen ("Learn to hear changes. Headphones on, phone away.")
2. Quick context capture: instrument (piano-first), self-assessed level, session length, commute days.
3. **Placement test:** a 3–4 minute mini Pocket Session sampling rungs 0–6 → places the user on the curriculum ladder. Design the "you're here" moment: the 9-rung ladder as an appealing map of the journey from "find the tonic" to "hear whole tunes."
4. Permission asks framed by value (notifications = practice window on commute days; Bluetooth/MIDI = later, only when they reach at-the-piano exercises).

### Journey 3 — Progress & the Ladder (retention surface)

- Home/hub: today's session, current rung, streak-as-minutes ("142 min this week"), weakest items.
- The Ladder: 9 rungs from Tonic Orientation → Whole Tunes; each rung shows its skills and mastery (SRS-based, not XP). This is the app's "map" — make it the emotional centerpiece of progress.
- Item detail: e.g., "Dominant ♭9 — 83% · often confused with #9" with a play/compare affordance.

### Journey 4 — Tune Workbench (differentiator; one or two hero screens are enough)

Blind transcription, productized: pick a standard → hear its changes in jazz-trio texture with the chart hidden → identify form, then landmark chords, then cell-by-cell — the chart *reveals itself* as you get sections right. Design the reveal mechanic: a real-book-style lead sheet that starts as empty bars and fills in as the user proves they hear each cell. Missed cells visibly flow into tomorrow's Pocket Session queue.

### Journey 5 — At the Piano: instrument call & response (concept-level only for now)

*(Replaced 2026-07 — was a voice/singing mode; voice interaction is now out of scope entirely.)* Evening companion mode at the instrument: app plays a phrase/voicing → user plays it back on a MIDI keyboard (Bluetooth/USB) → note-for-note feedback, with the same comparison-loop spirit on misses. Design questions: the listening/played-back feedback moment (encouraging, not clinical — the "never a red X" principle applies), and MIDI pairing UX that doesn't feel like printer setup. One hero screen + the awaiting-input state.

## Screen inventory (minimum set)

1. Home / today's session
2. Pocket Session: pre-session → playing → answer reveal → miss/comparison → complete (5–6 states)
3. Lock screen / Dynamic Island treatment
4. Onboarding (3–4 screens) + placement result ("you're here" ladder moment)
5. Ladder / progress hub + one item-detail
6. Tune Workbench: tune list + the self-revealing lead sheet
7. (Stretch) At the Piano: awaiting-input / played-back feedback state

## Constraints & platform notes

- iOS 17+, SwiftUI. Native components welcome; custom where the audio-first concept demands it.
- Must feel great one-handed on a small phone (train grip); test layouts at iPhone mini width.
- Background audio + lock-screen controls are core, not an afterthought — the lock screen *is* a primary UI.
- Offline-first, no accounts. One-time purchase (no paywall screens needed yet; no subscription upsell patterns).
- Music rendering: chord symbols (G7♭9, Fmaj7♯11), mini keyboard diagrams, voice-leading contours, and occasional lead-sheet bars — no full engraved notation, and nothing notational during listening/answering (principle 7).

## Out of scope for this pass

Settings, purchase flow, Android/web, social/leaderboards (deliberately excluded from product), notification content design.

## Deliverables wanted

1. A mood/direction take (2 directions max) on the "late-night jazz club × modern instrument" feel.
2. High-fidelity mockups of Journeys 1–3 (all Pocket Session states), medium-fidelity for 4, concept sketch for 5.
3. The visual language system for musical content: keys, chord qualities, tension colors, mastery states — plus the two reveal-moment visualizations: a **mini keyboard diagram** (shows the actual notes of a revealed voicing — how pianists think) and a **guide-tone contour** (two dots stepping through the 3rd/7th voice-leading of a progression). Both appear only post-answer, per principle 7.
4. A clickable flow of the Pocket Session loop if possible — the pacing/choreography (context → question → gap → reveal) is the product; even a rough animated sequence of that rhythm is more valuable than static polish.

## Background docs

- `docs/RESEARCH.md` — pedagogy findings + competitive landscape (why these journeys).
- `docs/CONCEPT.md` — full product concept, curriculum ladder, architecture.
