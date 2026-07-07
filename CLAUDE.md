# Jazzear Development Guidelines

> Last reviewed: 2026-07-07.

> ## ⚠️ PROJECT STATUS: GREENFIELD — docs only, no code yet
>
> The repo currently contains product docs (`docs/`) and this file. The
> architecture below is the **intended** design, carried over from lessons
> learned on [intrada](https://github.com/jonyardley/intrada) (same
> Crux + SwiftUI stack). As scaffolding lands, update this banner and replace
> "planned" sections with reality. When docs and code disagree, the code is
> reality and this file has a bug — fix it.

## Project Overview

Jazzear ("Changes" is the product working title) is a **hands-free jazz ear
training app for iOS**. It trains functional hearing — notes against a key
center, jazz chord qualities via decomposition (bass → 3rd → 7th → colors),
guide tones, and progression cells — in short, audio-first sessions designed
for a commute (phone in pocket, headphones on). A spaced-repetition engine
drives what the user practices each day.

**Product principles** (full rationale in `docs/CONCEPT.md` and
`docs/RESEARCH.md`):

- **Audio-first, screen-optional.** Every core exercise must be completable
  with headphones only. Lock screen / Dynamic Island is a primary UI.
- **Functional over intervallic.** Sounds are always heard against an
  established key context (cadence/drone). No acontextual interval quizzing.
- **Errors are the curriculum.** A miss triggers comparison-replay (the
  Banacos loop), never just "wrong, next."
- **Honest gamification.** Streaks measure minutes of real work; mastery is
  SRS-based, no XP/lives/leagues.
- **Offline-first, no accounts, one-time purchase.** No server, no sign-in.

The design brief for the UX is `docs/DESIGN_BRIEF.md`; mockups from Claude
Design land in `design/` and become the reference once they exist.

## Project Structure (planned)

```text
crates/
  jazzear-core/          # Pure Crux core — ALL logic, no I/O:
                         #   theory engine (keys, chords, voicings, cells)
                         #   exercise generator, session state machine
                         #   SRS scheduler, grading, pitch analysis
  jazzear-ffi/           # UniFFI / typegen bridge crate
ios/                     # SwiftUI shell (Xcode project via xcodegen)
  Jazzear/               #   app source; generated bindings under ios/generated/
design/                  # Claude Design mockups + design system reference
docs/                    # Product docs: CONCEPT, RESEARCH, DESIGN_BRIEF, roadmap
```

## Tech Stack (planned)

- **Rust** stable; **crux_core**, serde, thiserror, ulid, chrono
- **Bindings**: generated Swift types (UniFFI + typegen, as on intrada) —
  regenerated as part of the build, never hand-edited
- **iOS**: SwiftUI, iOS 17+, xcodegen; `@Observable` store wrapping the core
- **Audio out**: `AVAudioEngine` + sampler (SoundFont piano/bass/ride),
  driven entirely by core-emitted score events
- **Audio in** (later phases): `AVAudioEngine` mic tap → raw buffers →
  core-side pitch detection (YIN/pYIN in Rust)
- **Persistence**: GRDB (SQLite) executing typed storage effects; `crux_kv`
  for small singletons only
- **Speech**: `AVSpeechSynthesizer` for spoken answers in Pocket Sessions
- **Task runner**: `just` (mirror intrada's justfile recipes: `just ios`,
  `just ios-run`, `just ios-gen`, `just ios-logs`)

## Commands

To be established with the first scaffold. Target set (names match intrada so
muscle memory transfers):

```bash
cargo fmt --check          # must pass before commit AND push
cargo clippy -- -D warnings
cargo test                 # core is pure — the whole brain tests in seconds
just ios                   # regen bindings if core changed + open Xcode
just ios-run               # build + launch on simulator + screenshot
```

Run fmt + clippy locally *before pushing*, not just before committing —
a red CI roundtrip costs more than the local check.

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User/Audio → Events → jazzear-core (Rust) → Effects (PlayScore, Storage,
                                             Speak, Render, …) → Shell → I/O
```

1. **Core owns ALL logic.** Music theory, exercise generation, session
   choreography, SRS scheduling, answer grading, pitch analysis. If you're
   tempted to write logic in Swift, it belongs in `jazzear-core` as an
   `Event`/`Command`.
2. **Shells are dumb pipes.** The Swift shell renders `ViewModel`, sends
   `Event`s, and fulfils effects: realize a `PlayScore` with the sampler,
   execute a typed storage op, speak a string, stream mic buffers. No domain
   types, no musical decisions, no validation in Swift.
3. **The core is deterministic.** An exercise is a pure function of
   (curriculum state, SRS state, RNG seed). No wall-clock or RNG calls inside
   core logic — time and seeds arrive via events/effects. This is what makes
   the whole trainer testable without a simulator.

### Audio boundary (the project-specific rule)

- The core decides **what** to play and **when in musical terms**: an abstract
  score (notes/voicings with onsets, durations, tempo, instrument roles) plus
  session choreography (context → question → thinking gap → reveal).
- The shell owns **precise realization**: sample-accurate scheduling on
  `AVAudioEngine`, audio session config, interruption handling (calls, train
  announcements → auto-pause event back to core).
- The shell never invents, transposes, or re-voices musical content. If the
  sound is wrong, the bug is in core (or the SoundFont), not in Swift.

### Persistence & offline invariants

Offline-first with **no server at all** — simpler than intrada, but keep the
door open:

1. On-device SQLite is the only source of truth. No network on any core path.
2. Every persisted entity carries `updated_at` + soft-delete `deleted_at`
   from day one (sync-ready if a backup/sync tier ever ships); client-minted
   ulids are canonical ids.
3. Storage migrations are append-only, forward-only, additive by default.
   **The device is the only copy of the user's practice history** — a
   destructive migration is unrecoverable. Every migration ships with a test
   that a DB populated at the previous version migrates with data intact.
4. A failed local write is never a silent success — storage effects resolve
   real failure outputs and the core surfaces them to a UI state.

### Swift shell rules (inherited from intrada, non-negotiable)

- **Bindings are a build precondition, never source.** Never hand-edit
  generated Swift; fix the Rust type and regenerate. A diff editing generated
  code is a blocker.
- **`@Observable`, not `ObservableObject`.** Store is `@Observable
  @MainActor`; effect handlers run off the main actor, hop back to resolve.
- **`try!` is banned like `unwrap()`.** No force-unwraps / `as!` without
  written justification. FFI and codec calls return real errors — handle and
  **surface** them (a swallowed bridge error is a silent no-op bug).
- **Surface, don't swallow — at every layer.** Every `ViewModel.error` has a
  UI surface; never fire a success haptic or dismiss a sheet before the core
  confirms.
- **Spacing, radius, colour, type are tokens, not literals** — a `Theme.swift`
  with `Jazzear*` token namespaces, from the first screen.
- **Quality is per-screen, not deferred**: snapshot test, VoiceOver labels,
  Dynamic Type with each screen — plus, for this app, **a
  screen-off/locked-phone pass**: every Pocket Session interaction must be
  verified with the display off.

## Code Style

- Rust stable. `cargo fmt` + `cargo clippy -- -D warnings` must pass.
- No `unwrap()` without justification.
- Prefer well-established libraries over custom implementations — except the
  music theory engine, which is core domain and stays first-party.

### Comments

Default to **no comments**. Self-explanatory code with well-named identifiers
beats commented code. A comment is justified ONLY in three buckets;
everything else gets deleted:

1. **Section headers** in a large file — one-line dividers (`// ── SRS ──`).
2. **Unusual/unreasonable things** — a non-obvious WHY: hidden constraint,
   workaround for a specific bug, framework quirk. Cite concretely (issue
   number, doc link, `BUG:` tag).
3. **Hacky/tactical code needing rework** — `// HACK(#N): …` tied to a
   tracked issue. A bare `// TODO` with no issue is not acceptable.

Never: restating WHAT the code does, narrating self-evident structure,
referencing the current task/PR, hedging without an issue. `///` doc comments
get the same treatment. Two-line cap as a smell test: longer usually means
"should be a function name, a type, or a CLAUDE.md entry."

## Testing

- **The core is the product — test it like one.** Theory engine, exercise
  generator, SRS scheduler, session state machine, and grading are pure Rust:
  edge cases, property-style checks where natural (e.g. "every generated
  voicing contains the quality-defining 3rd and 7th"), and deterministic
  replay via fixed seeds.
- **Bridge-crossing types need a real round-trip test as a build
  precondition** — a stub bridge can't catch a bincode wire break (intrada
  #846). Extend the round-trip helper to every `Event`/`Effect`/`ViewModel`
  payload before it's wired to a screen.
- **Audio choreography** is tested at the effect level: given a session plan,
  assert the sequence/timing of emitted `PlayScore`/`Speak` effects — no
  simulator or real audio needed.
- Snapshot tests per screen once UI lands (one pinned device + scale,
  load-bearing states only, optimize PNGs before committing — see intrada's
  snapshot hygiene rules if the suite grows).
- Skipping tests: say so explicitly in the PR description with the reason.
  "All existing tests pass" is not coverage for new code.

## Project-specific gotchas

Seeded from intrada scar tissue (same stack); grow this list as Jazzear earns
its own.

### JSON-only serde attrs break the Crux bincode FFI bridge

The shell exchanges `Event`/`Effect`/`ViewModel` with the core as positional
bincode (non-self-describing). serde attributes with JSON-only semantics
(`deserialize_with` helpers assuming key-presence, `skip_serializing_if` on
non-trailing fields) silently misalign the byte stream — the symptom is a
**silent no-op**, not a crash. If format-specific behaviour is needed, branch
on `Deserializer::is_human_readable()`. Cover every bridge-crossing type with
a real-bridge round-trip test.

### Shared-simulator rule (esp. in a git worktree)

The iOS Simulator and `CoreSimulatorService` are machine-global. Create
worktree-scoped sims (`xcrun simctl create …`) and target by UDID; never run
global sim resets blind (`simctl shutdown all`, `erase`, `killall …
CoreSimulatorService`). If a sim/test session you didn't start is active,
pause and ask rather than kill it.

### `option_env!` needs `cargo:rerun-if-env-changed`

Any build-script/env-read macro site must pair with
`println!("cargo:rerun-if-env-changed=NAME")` or rebuilds silently use stale
values.

### Audio-specific traps to respect from day one

- **Audio session category/options are load-bearing**: Pocket Sessions need
  `.playback` with background audio entitlement, correct interruption + route
  change handling (headphones unplugged on a train = auto-pause, not blast
  from the speaker).
- **Speech + music mixing**: spoken answers duck the sampler, not fight it —
  decide the ducking policy in one place in the shell.

## Workflow

Match ceremony to scope. Default to less; escalate only when work demands it.

- **Tier 1 — just do it**: bug fixes, copy, style tweaks, renames, lint
  fixes, single-file refactors, dep bumps, doc updates. No plan mode, no spec.
- **Tier 2 — plan mode** (default for feature work): new screen/exercise type
  following existing patterns, new core handler, new field on an entity. For
  UI work: check `docs/DESIGN_BRIEF.md` / `design/` first, then plan, then
  implement.
- **Tier 3 — lightweight spec** (rare; architectural): net-new top-level
  features, FFI bridge changes, storage schema changes, the audio engine
  itself. ONE markdown doc in `docs/specs/<feature>.md` (~100–200 lines:
  problem, approach, key decisions, open questions). Spec rides with the
  first implementation PR, not its own PR.
- **Sensitivity override**: storage schema/migrations and FFI contract
  changes go up at least one tier regardless of size.
- **Decision rule**: unsure between tiers → go one lighter; drift up if scope
  expands.

### Always

1. Check `docs/roadmap.md` — work should map to a roadmap item; no item =
   discuss first.
2. Never push to main. Feature branch + PR.
3. Keep both docs and code honest: update `docs/roadmap.md` on completion,
   and update this file when architecture/patterns change.

## Known Tech Debt

None yet — keep it that way by recording entries here the moment a shortcut
ships.
