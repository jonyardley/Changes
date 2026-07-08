import SharedTypes
import SwiftUI

/// Functional Pocket Session surface for the M2 engine: every state
/// reachable and tokened, pixel-close polish deferred to M3 (design 1a/2b
/// canvases). Manual pacing throughout — nothing advances without a tap.
struct RootView: View {
  @Environment(Store.self) private var store

  var body: some View {
    VStack(spacing: 0) {
      header
      Spacer()
      centre
      Spacer()
      controls
    }
    .padding(.horizontal, ChangesSpacing.screenPadding)
    .background(ChangesColor.background.ignoresSafeArea())
  }

  private var vm: ViewModel? { store.viewModel }

  // ── Header ────────────────────────────────────────────────────────────

  private var header: some View {
    HStack {
      if let vm, vm.phase != .pre, !vm.keyName.isEmpty {
        Text("in \(vm.keyName)")
          .font(ChangesFont.musicKeyBadge)
          .foregroundStyle(ChangesColor.textPrimary)
          .padding(.horizontal, 14)
          .padding(.vertical, 6)
          .background(Capsule().fill(ChangesColor.surface))
          .accessibilityLabel("Key of \(vm.keyName)")
      } else {
        Text("Changes")
          .changesOverline()
          .accessibilityAddTraits(.isHeader)
      }
      Spacer()
      if let vm, vm.phase != .pre, vm.phase != .recap {
        Text("\(vm.itemNumber) / \(vm.totalItems)")
          .font(ChangesFont.uiCounter)
          .foregroundStyle(ChangesColor.textTertiary)
      }
    }
    .padding(.top, 8)
  }

  // ── Centre content ────────────────────────────────────────────────────

  @ViewBuilder
  private var centre: some View {
    VStack(spacing: 24) {
      if let vm {
        if vm.paused {
          pausedCard
        } else {
          switch vm.phase {
          case .pre: preContent
          case .listening: listeningContent
          case .gap: gapContent
          case .reveal: revealContent(vm)
          case .compare: compareContent(vm)
          case .recap: recapContent(vm)
          }
        }
        if let error = vm.error ?? store.error {
          Text(error)
            .font(ChangesFont.uiBody)
            .foregroundStyle(ChangesColor.Tension.flat9)
            .multilineTextAlignment(.center)
        }
        if store.degraded {
          Text("storage unavailable — this session won't be remembered")
            .font(ChangesFont.uiCounter)
            .foregroundStyle(ChangesColor.Tension.flat13)
        }
      }
    }
  }

  private var preContent: some View {
    VStack(spacing: 16) {
      Text("Today")
        .changesOverline()
      Text("scale degrees, one key")
        .font(ChangesFont.musicAccentLine)
        .foregroundStyle(ChangesColor.accent)
    }
  }

  private var listeningContent: some View {
    VStack(spacing: 16) {
      Text("Listen")
        .changesOverline()
      Text("establishing the key…")
        .font(ChangesFont.musicAccentLine)
        .foregroundStyle(ChangesColor.textSecondary)
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Listening. The cadence and question note are playing.")
  }

  private var gapContent: some View {
    VStack(spacing: 16) {
      Text("Name it")
        .changesOverline()
      Text("in your head — take your time")
        .font(ChangesFont.uiBody)
        .foregroundStyle(ChangesColor.textTertiary)
    }
  }

  private func revealContent(_ vm: ViewModel) -> some View {
    VStack(spacing: 12) {
      Text("It was")
        .changesOverline()
      if let answer = vm.answer {
        Text(answer.label)
          .font(ChangesFont.musicChordSymbol())
          .tracking(-90 * 0.02)
          .foregroundStyle(ChangesColor.textPrimary)
          .accessibilityLabel("The degree was \(answer.label)")
        Text(answer.resolution)
          .font(ChangesFont.musicAccentLine)
          .foregroundStyle(ChangesColor.accent)
          .accessibilityLabel("Resolves \(answer.resolution)")
      }
    }
  }

  private func compareContent(_ vm: ViewModel) -> some View {
    VStack(spacing: 20) {
      Text("Hear the difference")
        .changesOverline()
      if let compare = vm.compare {
        HStack(spacing: 16) {
          compareCard(label: compare.missed, active: !compare.playingTwin, caption: "you missed")
          compareCard(label: compare.twin, active: compare.playingTwin, caption: "its twin")
        }
      }
    }
  }

  private func compareCard(label: String, active: Bool, caption: String) -> some View {
    VStack(spacing: 8) {
      Text(label)
        .font(ChangesFont.musicHeadline(44))
        .foregroundStyle(ChangesColor.textPrimary)
      Text(active ? "playing" : caption)
        .changesOverline()
    }
    .frame(maxWidth: .infinity, minHeight: 140)
    .background(
      RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
        .fill(ChangesColor.surface)
        .overlay(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
            .strokeBorder(active ? ChangesColor.accent : ChangesColor.hairline)
        )
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label), \(active ? "playing now" : caption)")
  }

  private func recapContent(_ vm: ViewModel) -> some View {
    VStack(spacing: 16) {
      Text("Session complete")
        .changesOverline()
      Text("nice ears tonight.")
        .font(ChangesFont.musicHeadline())
        .foregroundStyle(ChangesColor.textPrimary)
      if let recap = vm.recap {
        HStack(spacing: 28) {
          recapStat(value: recap.got, label: "heard")
          recapStat(value: recap.missed, label: "to revisit")
        }
      }
    }
  }

  private func recapStat(value: UInt32, label: String) -> some View {
    VStack(spacing: 4) {
      Text("\(value)")
        .font(ChangesFont.uiStat)
        .foregroundStyle(ChangesColor.textPrimary)
      Text(label)
        .changesOverline()
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(value) \(label)")
  }

  private var pausedCard: some View {
    VStack(spacing: 12) {
      Text("Paused")
        .changesOverline()
      Text("take your time — the key will still be here")
        .font(ChangesFont.musicAccentLine)
        .foregroundStyle(ChangesColor.textSecondary)
        .multilineTextAlignment(.center)
      Text("resuming replays this item")
        .font(ChangesFont.uiCounter)
        .foregroundStyle(ChangesColor.textTertiary)
    }
    .padding(24)
    .background(
      RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
        .fill(ChangesColor.surface)
        .overlay(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
            .strokeBorder(ChangesColor.Tension.flat13)
        )
    )
  }

  // ── Controls (the deliberate taps) ────────────────────────────────────

  @ViewBuilder
  private var controls: some View {
    if let vm {
      Group {
        if vm.paused {
          tapZone("resume", accent: true) { store.send(.tapResume) }
        } else {
          switch vm.phase {
          case .pre:
            tapZone(vm.isLoading ? "loading…" : "start today's session", accent: true) {
              startSession()
            }
            .disabled(vm.isLoading)
          case .listening:
            tapZone(vm.isPlaying ? "listening…" : "", accent: false) {}
              .disabled(true)
              .opacity(0.25)
          case .gap:
            tapZone("tap to reveal", accent: true) { store.send(.tapReveal) }
          case .reveal:
            answerZones
          case .compare:
            tapZone("I hear it — continue", accent: true) { store.send(.exitCompare) }
          case .recap:
            tapZone("go again", accent: true) { startSession() }
          }
        }
      }
      .padding(.bottom, 16)
    }
  }

  private var answerZones: some View {
    HStack(spacing: 12) {
      answerZone("Got it", event: .gradeGotIt)
      answerZone("Missed it", event: .gradeMissedIt)
    }
  }

  private func answerZone(_ label: String, event: Event) -> some View {
    Button {
      store.send(event)
    } label: {
      Text(label)
        .font(ChangesFont.uiButton)
        .foregroundStyle(ChangesColor.textPrimary)
        .frame(maxWidth: .infinity, minHeight: ChangesSpacing.answerZoneHeight)
        .background(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
            .fill(ChangesColor.surface)
            .overlay(
              RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
                .strokeBorder(ChangesColor.hairline)
            )
        )
    }
    .accessibilityHint("Grades this item and moves on")
  }

  private func tapZone(_ label: String, accent: Bool, action: @escaping () -> Void) -> some View {
    Button(action: action) {
      Text(label)
        .font(ChangesFont.uiButton)
        .foregroundStyle(ChangesColor.textPrimary)
        .frame(maxWidth: .infinity, minHeight: ChangesSpacing.answerZoneHeight)
        .background(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
            .fill(ChangesColor.surface)
            .overlay(
              RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
                .strokeBorder(accent ? ChangesColor.accentBorder : ChangesColor.hairline)
            )
        )
    }
    .changesAccentGlow()
  }

  private func startSession() {
    // Shell-provided entropy + clock: the core is deterministic given both.
    let nowMs = Int64(Date.now.timeIntervalSince1970 * 1000)
    store.send(.startSession(seed: UInt64(bitPattern: nowMs), nowMs: nowMs))
  }
}
