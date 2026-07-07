# Jazz Ear Training — Research Findings

*Compiled 2026-07-07. Two research passes: (1) pedagogy of jazz ear training, (2) iOS app landscape.*

## 1. Pedagogy: what actually works

### The central finding

Jazz ear training literature splits into two camps: **intervallic** (drill intervals/chord qualities in isolation — the conservatory and David Lucas Burge model) and **functional/contextual** (hear every note relative to a key center — Alain Benbassat, Bruce Arnold, Charlie Banacos). The modern consensus strongly favors functional-first. Gary Karpinski's research (via Timothy Chenette, Music Theory Online): *"a preponderance of experimental evidence shows little connection between the ability to identify intervals acontextually and the ability to do so in a tonal context."* You get good at the quiz, not at music.

What experienced jazz musicians rank highest for transfer to real outcomes:

1. **Transcription** / learning tunes and language from records
2. **Singing everything before playing it** (audiation — "if you can't sing it, you can't play it"; Tristano made students sing solos before touching the instrument)
3. **Functional hearing** of scale degrees and progression *chunks*
4. Playing with people/records

Interval and chord-quality drills are endorsed only as *supporting* fundamentals in small doses.

### Skill-specific findings

- **Functional/scale-degree hearing** — the foundation. Benbassat format: cadence establishes key → one note played → identify its scale degree by how it wants to resolve. Bruce Arnold's "One Note": memorize all 12 chromatic degree "colors" against a key center; train recognition *and* sung production in parallel.
- **Chord qualities, jazz-style** — learned *decompositionally*, not as gestalt flashcards: hear the bass note → major-or-minor 3rd → 7th type → extensions → alterations. Jazzadvice emphasizes hearing individual chord-tone *colors* (what a #9 sounds like against a dominant) — that's what transfers to improvising.
- **Guide tones (3rds & 7ths)** — hearing the stepwise voice-leading line through ii-V-I is treated as the master key to hearing harmony move, and the bridge to comping.
- **Progressions as chunks** — Jerry Coker's *Hearin' the Changes*: codify progression cells (ii-V-I, turnarounds, I-vi-ii-V, back-door dominants, blues variants, rhythm changes) and link each to named standards you already know. Chenette's research confirms skilled harmonic listeners work by (a) tracking the bass line and (b) recognizing familiar progression chunks — not computing chords one at a time.
- **Bass-line hearing** — the practical entry point to picking up tunes ("listen to what the bass player is doing").
- **Error-driven learning (Banacos pattern)** — on a miss, oscillate between the cadence and the missed note until the relationship clarifies. Wrong answers should trigger comparison/replay, not just "incorrect, next."

### Session design

- **Short daily sessions beat long infrequent ones.** 10–20 min/day minimum; four 5-minute sessions outperform one 20-minute session; ears fatigue fast. Open Studio's flagship course is literally "5-Minute Ear Training." → Validates a commute-length format.
- **Spaced repetition is a natural fit** (degree colors, chord qualities, progression cells are ideal SRS items) and essentially absent from commercial apps — players resort to Anki decks (AnkiJazz).
- **Gamification pitfalls** (Duolingo literature): streak-preservation via trivial lessons, easy loops giving "illusion of mastery," XP-chasing avoiding hard material. Counters: reward accuracy on *hard* items and production (singing), streaks measured in minutes-of-real-work, always tie drills back to real music.

### Synthesized progression ladder (jazz pianist)

- **Rung 0 — Tonic orientation:** find/sing Do of anything; resolve any note to Do.
- **Rung 1 — Diatonic degrees, major:** identify 1–7 after a cadence; then *sing* requested degrees.
- **Rung 2 — Minor keys + chromatic degrees:** all 12 note-colors; contextual intervals (from tonic → between degrees).
- **Rung 3 — Melodies:** sing back short phrases; degree-dictation of tune fragments; one standard melody/week by ear.
- **Rung 4 — Chord qualities via decomposition:** bass → 3rd → 7th; triads + maj7/m7/7/m7b5/dim7; then inversions/spreads.
- **Rung 5 — Guide tones & ii-V-I:** hear 3rd/7th lines resolve; ii-V-I major/minor; resolved vs deceptive dominants.
- **Rung 6 — Bass lines & progression cells:** track roots through choruses; recognize turnaround variants, blues forms, rhythm changes, back-door, tritone subs — each linked to named standards.
- **Rung 7 — Colors & voicings (pianist-specific):** extensions/alterations as single colors (9, 13, b9, #9, #11, b13); upper-structure triads; voicing types (rootless A/B, quartal, block).
- **Rung 8 — Whole-tune fluency:** form + big harmonic landmarks in 1–2 listens; transcribe by singing first; rhythmic ear work against records.
- **Always-on:** sing before playing; short daily sessions; SRS review of passed items; end every session touching real music.

## 2. App landscape: the gap

**Bottom line: jazz progression/chord-quality recognition + audio-first hands-free + offline + SRS is served by zero apps today.**

| App | Strength | Why it doesn't cover this |
|---|---|---|
| EarMaster | Real jazz workshops (chord qualities, progressions, swing) | Tap/notation-heavy UX, dated; not hands-free |
| Functional Ear Trainer | Canonical Benbassat method; has a hands-free mode | Melody-degree only; no harmony; plateaus |
| Sonofield (2025) | Best hands-free UX ("Pocket Mode" for commutes); drone-grounded | No chords at all |
| Toned Ear | Best chord-progression trainer available | Block-chord piano, not jazz texture; not hands-free; buggy app |
| Chet | Real recorded audio, jazz-aware scales, call-and-response on instrument | Instrument-input = hands-on; leans absolute over functional |
| Tenuto / Perfect Ear / Complete Ear Trainer / MyEarTraining / ToneGym | Polished isolated drilling | The exact approach pedagogy says doesn't transfer |
| iReal Pro | De facto jazz practice companion | No quiz/assessment layer — players hack blind-transcription workflows around it |

**Documented gaps:**

1. Jazz harmony in context: ii-V-I vs back-door vs bird changes in realistic combo texture; alterations (b9/#9/#11/b13); upper structures; rootless/shell voicings rather than root-position block piano.
2. Hands-free harmony training — completely empty lane.
3. The transcription bridge: graded ladder from clean synthetic → stylized combo audio → real-recording excerpts.
4. Singing/mic as answer input for jazz (e.g., "sing the 3rd of this chord," guide-tone lines).
5. True spaced repetition.
6. Pricing: market resents subscriptions here (ToneGym backlash, FET's move); one-time purchase is the friendly position.

**Interaction paradigms worth borrowing:** cadence-then-target (FET); drone-grounded listening (Sonofield); auto-paced audio quiz with thinking gap + spoken answer (Sonofield Pocket Mode); call-and-response on MIDI/instrument (Chet); mic-graded singing (SingTrue/EarMaster); blind-transcribe-then-verify (iReal Pro DIY workflow); SRS decay scheduling (AnkiJazz).

## Key sources

Pedagogy: Jazzadvice (ear training method, hearing chord changes, singing, transcription, Clark Terry), Benbassat/Functional Ear Trainer, Charlie Banacos (miles.be; Kordis thesis), Bruce Arnold (muse-eek), Chenette "What Are the Truly Aural Skills?" (MTO 27.2), Coker *Hearin' the Changes*, Hal Galper *Forward Motion*, Gordon MLT, Open Studio 5-Minute Ear Training, Learn Jazz Standards, Musical U / ToneDear session-design articles, DAS3H spaced-practice scheduling (arXiv 1905.06873).

Apps: App Store listings and reviews for the apps above; jazzguitar.be forum threads (functional vs intervallic; best ways of ear training); Jon Mellman's iReal Pro ear-training workflow; AnkiJazz; Sonofield & ToneGym reviews.
