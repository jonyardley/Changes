import SharedTypes
import SwiftUI

/// The Pocket Session, pixel-close to design 1a/2b/4b: every state from
/// pre-session to recap, manually paced — nothing advances without a tap.
struct RootView: View {
  @Environment(Store.self) private var store
  @State private var minutes = 18

  /// Duration pill → session length (≈90s per manually-paced item).
  private var maxItems: UInt32 {
    switch minutes {
    case 10: 7
    case 25: 17
    default: 12
    }
  }

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
  private var inSession: Bool {
    guard let vm else { return false }
    return vm.phase != .pre && vm.phase != .recap
  }

  // ── Header: wordmark / key badge · counter · pause ────────────────────

  private var header: some View {
    HStack(spacing: 12) {
      if let vm, inSession, !vm.keyName.isEmpty {
        HStack(spacing: 6) {
          Circle().fill(ChangesColor.Quality.maj7).frame(width: 6, height: 6)
          Text("in \(vm.keyName)")
            .font(ChangesFont.musicKeyBadge)
            .foregroundStyle(ChangesColor.textPrimary)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 6)
        .background(Capsule().fill(ChangesColor.surface))
        .accessibilityLabel("Key of \(vm.keyName)")
      } else {
        Text("Changes")
          .font(ChangesFont.uiOverline)
          .textCase(.uppercase)
          .tracking(13 * 0.22)
          .foregroundStyle(ChangesColor.textSecondary)
          .accessibilityAddTraits(.isHeader)
      }
      Spacer()
      if let vm, inSession {
        Text("\(vm.itemNumber) / \(vm.totalItems)")
          .font(ChangesFont.uiCounter)
          .foregroundStyle(ChangesColor.textTertiary)
        if vm.pause == .none {
          Button {
            store.send(.tapPause)
          } label: {
            Image(systemName: "pause.fill")
              .font(.system(size: 13))
              .foregroundStyle(ChangesColor.textSecondary)
              .frame(width: 34, height: 34)
              .background(Circle().fill(ChangesColor.surface))
          }
          .accessibilityLabel("Pause session")
        }
      }
    }
    .padding(.top, 8)
  }

  // ── Centre content per state ──────────────────────────────────────────

  @ViewBuilder
  private var centre: some View {
    VStack(spacing: 24) {
      if let vm {
        switch vm.pause {
        case .user: pausedCard(vm)
        case .interrupted: interruptedBanner
        case .none:
          switch vm.phase {
          case .pre: preContent
          case .context: contextContent(vm)
          case .question: questionContent
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
    VStack(spacing: 22) {
      VStack(spacing: 8) {
        Text("Today")
          .changesOverline()
        Text("scale degrees, one key")
          .font(ChangesFont.musicAccentLine)
          .foregroundStyle(ChangesColor.accent)
      }
      PlayButton(
        label: vm?.isLoading == true ? "loading…" : "begin",
        enabled: vm?.isLoading != true
      ) {
        startSession()
      }
      DurationPills(minutes: $minutes)
    }
  }

  private func contextContent(_ vm: ViewModel) -> some View {
    VStack(spacing: 18) {
      Text("Listen")
        .changesOverline()
      LevelBars(animating: vm.isPlaying)
      Text("establishing \(vm.keyName)")
        .font(ChangesFont.musicAccentLine)
        .foregroundStyle(ChangesColor.textSecondary)
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Listening. The cadence is establishing \(vm.keyName).")
  }

  private var questionContent: some View {
    VStack(spacing: 18) {
      Text("Degree of this note")
        .changesOverline()
      QuestionPulse()
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("The question note is playing. Which degree is it?")
  }

  private var gapContent: some View {
    VStack(spacing: 12) {
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
          .font(ChangesFont.musicChordSymbol(84))
          .tracking(-84 * 0.02)
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
          compareCard(label: compare.missed, active: !compare.playingTwin)
          compareCard(label: compare.twin, active: compare.playingTwin)
        }
        Text("hear it pull home — that's the color")
          .font(ChangesFont.musicAccentLine)
          .foregroundStyle(ChangesColor.accent)
      }
    }
  }

  private func compareCard(label: String, active: Bool) -> some View {
    VStack(spacing: 10) {
      Text(label)
        .font(ChangesFont.musicHeadline(44))
        .foregroundStyle(ChangesColor.textPrimary)
      Text(active ? "playing" : "next")
        .changesOverline()
    }
    .frame(maxWidth: .infinity, minHeight: 150)
    .background(
      RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
        .fill(ChangesColor.surface)
        .overlay(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
            .strokeBorder(active ? ChangesColor.accent : ChangesColor.hairline)
        )
    )
    .shadow(
      color: active ? ChangesGlow.accentColor : .clear, radius: ChangesGlow.accentRadius / 2
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label), \(active ? "playing now" : "plays next")")
  }

  private func recapContent(_ vm: ViewModel) -> some View {
    VStack(spacing: 20) {
      VStack(spacing: 8) {
        Text("Session complete")
          .changesOverline()
        Text("nice ears tonight.")
          .font(ChangesFont.musicHeadline())
          .foregroundStyle(ChangesColor.textPrimary)
      }
      if let recap = vm.recap {
        VStack(spacing: 0) {
          LedgerRow(label: "reviewed", value: "\(recap.got + recap.missed)")
          LedgerRow(label: "heard right", value: "\(recap.got)", glow: recap.missed == 0)
          LedgerRow(label: "to revisit", value: "\(recap.missed)")
        }
        pianoCard(recap)
      }
    }
  }

  private func pianoCard(_ recap: RecapView) -> some View {
    VStack(alignment: .leading, spacing: 6) {
      Text("Tonight at the piano")
        .changesOverline()
      Text(
        recap.missed > 0
          ? "play the \(recap.missed) you missed against the key — hear each pull home"
          : "play today's degrees against the key, eyes closed"
      )
      .font(ChangesFont.musicAccentLine)
      .foregroundStyle(ChangesColor.textPrimary)
    }
    .frame(maxWidth: .infinity, alignment: .leading)
    .padding(18)
    .background(
      RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
        .fill(ChangesColor.surface)
        .overlay(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
            .strokeBorder(ChangesColor.accentBorder)
        )
    )
  }

  private func pausedCard(_ vm: ViewModel) -> some View {
    VStack(spacing: 12) {
      Text("Paused")
        .changesOverline()
      Text("take your time — \(vm.keyName) will still be here")
        .font(ChangesFont.musicAccentLine)
        .foregroundStyle(ChangesColor.textSecondary)
        .multilineTextAlignment(.center)
      Text("progress saved · session resumes mid-item")
        .font(ChangesFont.uiCounter)
        .textCase(.uppercase)
        .tracking(1)
        .foregroundStyle(ChangesColor.textTertiary)
    }
    .padding(24)
    .frame(maxWidth: .infinity)
    .background(
      RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
        .fill(ChangesColor.surface)
        .overlay(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCardLarge)
            .strokeBorder(ChangesColor.hairline)
        )
    )
  }

  private var interruptedBanner: some View {
    VStack(spacing: 10) {
      Text("Other audio detected — holding")
        .font(ChangesFont.uiBodyMedium)
        .foregroundStyle(ChangesColor.textPrimary)
      Text("tap resume when it's quiet · replays the item")
        .font(ChangesFont.uiCounter)
        .foregroundStyle(ChangesColor.textTertiary)
    }
    .padding(20)
    .frame(maxWidth: .infinity)
    .background(
      RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
        .fill(ChangesColor.surface)
        .overlay(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
            .strokeBorder(ChangesColor.Tension.flat13)
        )
    )
    .accessibilityElement(children: .combine)
    .accessibilityLabel("Session held: other audio detected. Tap resume when it's quiet.")
  }

  // ── Controls (the deliberate taps; zones fade 25% ↔ 100%) ─────────────

  @ViewBuilder
  private var controls: some View {
    if let vm {
      Group {
        if vm.pause != .none {
          tapZone("resume", accent: true) { store.send(.tapResume) }
        } else {
          switch vm.phase {
          case .pre:
            EmptyView()
          case .context, .question:
            answerZones(vm, enabled: false).opacity(0.25)
          case .gap:
            tapZone("tap to reveal", accent: true) { store.send(.tapReveal) }
          case .reveal:
            answerZones(vm, enabled: true)
          case .compare:
            tapZone("I hear it — continue", accent: true) { store.send(.exitCompare) }
          case .recap:
            tapZone("go again", accent: true) { startSession() }
          }
        }
      }
      .animation(.easeInOut(duration: 0.3), value: vm.phase)
      .padding(.bottom, 16)
    }
  }

  private func answerZones(_ vm: ViewModel, enabled: Bool) -> some View {
    HStack(spacing: 12) {
      answerZone("Got it", event: .gradeGotIt, enabled: enabled)
      answerZone("Missed it", event: .gradeMissedIt, enabled: enabled)
    }
  }

  private func answerZone(_ label: String, event: Event, enabled: Bool) -> some View {
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
    .disabled(!enabled)
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
    store.send(.startSession(seed: UInt64(bitPattern: nowMs), nowMs: nowMs, maxItems: maxItems))
  }
}
