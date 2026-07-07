# Changes Development Guidelines

> Last reviewed: 2026-07-07.

> ## ⚠️ PROJECT STATUS: M0 complete — next M1 (audio spike, the go/no-go gate)
>
> What exists: product docs (`docs/`), a completed Claude Design pass
> (`design/` — read `design/README.md`), the implementation plan
> (`docs/specs/mvp-plan.md`, milestones M0–M6, decisions taken), the
> **Rust workspace with all quality gates live** (crux walking skeleton in
> `crates/changes-core`, strict lints, cargo-deny, CI, justfile — run
> `just setup` once per clone), and the **iOS scaffold** (xcodegen project
> in `ios/`, `Theme.swift` token sheet + bundled fonts, uniffi + facet
> typegen bindings proving `Event::Ping → ViewModel` through the real
> bridge, iOS CI job in CI OK's `needs`; `just ios-run` launches it).
> **Next: M1 (audio spike — the explicit go/no-go gate; exit criteria need
> a real device).** Architecture carries lessons from
> [intrada](https://github.com/jonyardley/intrada) (same Crux + SwiftUI
> stack). As code lands, update this banner and replace "planned" sections
> with reality. When docs and code disagree, the code is reality and this
> file has a bug — fix it.
>
> One superseded-docs note: `design/README.md` is a historical handoff record
> and still mentions spoken answers, auto-pacing, and the lock-screen layer;
> **voice I/O was cut 2026-07, and auto-pacing + the hands-free pocket layer
> were deferred 2026-07-07** — `docs/specs/mvp-plan.md` decisions 5, 9, 10
> are authoritative (aural reveal + on-screen answer; tap-paced; foreground
> only).

## Project Overview

Changes (repo formerly named "Jazzear", renamed 2026-07-07) is a **touch-driven,
audio-first jazz ear training app for iOS**. It trains functional hearing —
notes against a key center, jazz chord qualities via decomposition (bass →
3rd → 7th → colors), guide tones, and progression cells — in short, short
audio-first sessions for found time (commute, sofa), screen in hand,
headphones on. A spaced-repetition engine drives what the user practices
each day. A hands-free "pocket mode" (locked phone, earbud taps,
auto-pacing) is a deferred post-MVP exploration, not part of the initial
build.

**Product principles** (full rationale in `docs/CONCEPT.md` and
`docs/RESEARCH.md`):

- **Audio-first — audio is OUTPUT ONLY.** Music (cadences, chords, examples)
  plays out; the app never speaks and never listens. ALL input is touch on
  the screen. No TTS, no voice commands, no speech recognition — in any
  form, ever (the pre-recorded-voice-clip fallback idea was rejected
  2026-07-07).
- **User-paced — manual pacing (2026-07 decision).** Nothing happens without
  a deliberate tap: tap to start an item, tap to reveal the answer (the
  thinking gap is open-ended — no timer), tap to grade (the grade tap
  doubles as "next"). Auto-paced flow (timed reveal, auto-continue, passive
  "shadow mode") is deferred post-MVP — do NOT implement it initially.
- **Foreground-first MVP (2026-07 decision).** Used screen in hand. The
  hands-free "pocket mode" layer — background audio, lock screen / Now
  Playing / Live Activity, earbud remote commands — is one bundled deferral
  with auto-pacing; don't build any of it initially.
- **Functional over intervallic.** Sounds are always heard against an
  established key context (cadence/drone). No acontextual interval quizzing.
- **Errors are the curriculum.** A miss triggers comparison-replay (the
  Banacos loop), never just "wrong, next."
- **Honest gamification.** Streaks measure minutes of real work; mastery is
  SRS-based, no XP/lives/leagues.
- **Offline-first, no accounts, one-time purchase.** No server, no sign-in.

The design brief for the UX is `docs/DESIGN_BRIEF.md`. **The design reference
is `design/`** (Claude Design handoff — direction 1a "Blue Hour Console"):
`design/README.md` carries the token sheet, screen inventory, and the Pocket
Session state machine whose *playback* timings are **product spec**, not
illustration — but the prototype's pacing (timed thinking gap,
auto-continue) and its lock-screen/earbud layer are superseded by the
manual, foreground-first decisions (see the README's banner).
The active implementation plan is `docs/specs/mvp-plan.md`. Design tokens
map 1:1 into `Theme.swift` (`Changes*` namespaces) at scaffold time.

## Project Structure

```text
crates/
  changes-core/          # Pure Crux core — ALL logic, no I/O:  [exists]
                         #   theory engine (keys, chords, voicings, cells)
                         #   exercise generator, session state machine
                         #   SRS scheduler, grading, pitch analysis
  changes-ffi/           # UniFFI bridge + facet typegen codegen bin  [exists]
ios/                     # SwiftUI shell (xcodegen; project.yml)  [exists]
  Changes/               #   app source; generated bindings under ios/generated/
                         #   (gitignored — `just ios-gen` regenerates)
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
- **Instrument in** (later phases): CoreMIDI (Bluetooth LE / USB) → core
  events for played-answer grading; later, `AVAudioEngine` mic tap → raw
  buffers → core-side note detection for acoustic pianos. **No voice I/O in
  either direction** — no TTS, no sung-answer grading (decision 2026-07);
  the answer reveal is aural (resolution / decomposition playback) plus the
  on-screen answer
- **Persistence**: GRDB (SQLite) executing typed storage effects; `crux_kv`
  for small singletons only
- **Task runner**: `just` (mirror intrada's justfile recipes: `just ios`,
  `just ios-run`, `just ios-gen`, `just ios-logs`)

## Commands

```bash
just setup                 # once per clone: installs the pre-push hook
just ci                    # full CI mirror: fmt-check, clippy, test, deny, links
just pre-push              # fast gates (fmt-check + clippy + test) — the hook runs this
just fmt / clippy / test / deny / links   # individual gates
just ios / ios-run / ios-gen / ios-logs   # Xcode / sim launch / bindings / logs
```

`just ci` mirrors every CI gate (gitleaks is skipped with a warning if the
binary isn't installed — `brew install gitleaks`); the single required check
is **CI OK**. Run `just pre-push` locally *before pushing*, not just before
committing — a red CI roundtrip costs more than the local check (the hook
enforces this).

Lint policy (enforced, not aspirational): `clippy::unwrap_used` is **denied**
workspace-wide — a per-site `#[allow]` needs a WHY comment; tests are exempt
(`clippy.toml`). `expect_used`/`panic`/`todo`/`dbg!`/`print*` warn locally
and fail CI via `-D warnings`. The toolchain is pinned in
`rust-toolchain.toml`; supply-chain gates live in `deny.toml` (permissive
licenses only — closed-source app).

## Architecture (Non-Negotiables)

### Crux capabilities pattern

```text
User/Audio/MIDI → Events → changes-core (Rust) → Effects (PlayScore,
                                            Storage, Render, …) → Shell → I/O
```

1. **Core owns ALL logic.** Music theory, exercise generation, session
   choreography, SRS scheduling, answer grading, pitch analysis. If you're
   tempted to write logic in Swift, it belongs in `changes-core` as an
   `Event`/`Command`.
2. **Shells are dumb pipes.** The Swift shell renders `ViewModel`, sends
   `Event`s, and fulfils effects: realize a `PlayScore` with the sampler,
   execute a typed storage op, forward MIDI/remote-command events. No domain
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
  with `Changes*` token namespaces, from the first screen.
- **Quality is per-screen, not deferred**: snapshot test, VoiceOver labels,
  Dynamic Type with each screen. (A screen-off/locked-phone pass joins this
  list if/when the deferred pocket mode is built.)

## Code Style

- Rust stable. `just fmt-check` + `just clippy` must pass (clippy runs
  `--locked --workspace --all-targets` — always use the just recipe, a bare
  `cargo clippy` misses test code and lockfile drift).
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
  #846). Extend the round-trip helper
  (`assert_bincode_round_trip` in `crates/changes-core/src/test_support.rs`)
  to every `Event`/`Effect`/`ViewModel` payload before it's wired to a
  screen (effect payloads round-trip their operation types — the generated
  `*Ffi` enum has no `PartialEq`).
- **Audio choreography** is tested at the effect level: given a session plan,
  assert the sequence/timing of emitted `PlayScore`/timer effects — no
  simulator or real audio needed. (There is no `Speak` effect and never will
  be — no voice I/O.)
- Snapshot tests per screen once UI lands (one pinned device + scale,
  load-bearing states only, optimize PNGs before committing — see intrada's
  snapshot hygiene rules if the suite grows).
- Skipping tests: say so explicitly in the PR description with the reason.
  "All existing tests pass" is not coverage for new code.

## Project-specific gotchas

Seeded from intrada scar tissue (same stack); grow this list as Changes earns
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

- **Audio session category/options are load-bearing**: use `.playback` so
  sessions sound with the ring/silent switch on silent, and get interruption
  + route change handling right (headphones unplugged on a train =
  auto-pause, not blast from the speaker). Background-audio entitlement and
  lock-screen controls arrive with the deferred pocket mode, not the MVP.
- **No voice interface is a product decision, not an omission** (2026-07).
  Audio is output-only music — no voice audio of any kind, not even
  pre-recorded clips (rejected 2026-07-07); every input is a touch on the
  screen. Don't reintroduce `AVSpeechSynthesizer` for "quick"
  announcements, and don't add `SFSpeechRecognizer`/Siri intents for
  "convenient" voice control — the answer reveal is aural playback plus the
  on-screen answer by design (a Now Playing answer title arrives only with
  the deferred pocket mode, per mvp-plan decisions 3/10).

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
2. Never push to main — it's ruleset-protected (PRs only, no bypass, even
   for admins). Feature branch + PR; **squash-merge only**, and the PR title
   + body become the commit message, so write them as the permanent record.
   Merged branches auto-delete. Auto-merge is enabled — set it once CI is
   green rather than polling. The single required check is **"CI OK"**
   (`.github/workflows/ci.yml`): when adding CI jobs, wire them into
   `ci-ok`'s `needs` list; never touch branch protection.
3. Keep both docs and code honest: update `docs/roadmap.md` on completion,
   and update this file when architecture/patterns change.

## Known Tech Debt

None yet — keep it that way by recording entries here the moment a shortcut
ships.
