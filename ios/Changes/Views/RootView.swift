import SharedTypes
import SwiftUI

/// M1 audio-spike surface: one deliberate tap per phase (context → question
/// → reveal), answer shown at reveal only. Exists to run the M1 exit
/// criteria on a device; M3 replaces it with the Pocket Session screens.
struct RootView: View {
  @Environment(Store.self) private var store

  private var phaseLabel: String {
    guard let vm = store.viewModel else { return "" }
    if vm.paused { return "paused" }
    return switch vm.phase {
    case .idle: "tap to begin"
    case .context: "listen — establishing E♭"
    case .question: "quality of this chord?"
    case .reveal: "it was"
    }
  }

  private var tapPrompt: String {
    guard let vm = store.viewModel else { return "" }
    if vm.paused { return "tap to resume — replays this item" }
    return switch vm.phase {
    case .idle: "tap to start"
    case .context: "tap for the question"
    case .question: "tap to reveal"
    case .reveal: "tap for the next item"
    }
  }

  var body: some View {
    VStack(spacing: 24) {
      header

      Spacer()

      Text(phaseLabel)
        .changesOverline()

      if let answer = store.viewModel?.answer {
        Text(answer)
          .font(ChangesFont.musicChordSymbol())
          .tracking(-90 * 0.02)
          .foregroundStyle(ChangesColor.textPrimary)
          .accessibilityLabel("The chord was \(answer)")
      }

      if store.viewModel?.isPlaying == true {
        Text("playing…")
          .font(ChangesFont.uiCounter)
          .foregroundStyle(ChangesColor.accent)
      }

      if let error = store.error ?? store.viewModel?.error {
        Text(error)
          .font(ChangesFont.uiBody)
          .foregroundStyle(ChangesColor.Tension.flat9)
          .multilineTextAlignment(.center)
      }

      Spacer()

      tapZone
    }
    .padding(.horizontal, ChangesSpacing.screenPadding)
    .background(ChangesColor.background.ignoresSafeArea())
  }

  private var header: some View {
    HStack {
      Text("Changes")
        .changesOverline()
        .accessibilityAddTraits(.isHeader)
      Spacer()
      if let vm = store.viewModel, vm.phase != .idle {
        Text("item \(vm.itemNumber)")
          .font(ChangesFont.uiCounter)
          .foregroundStyle(ChangesColor.textTertiary)
      }
    }
    .padding(.top, 8)
  }

  private var tapZone: some View {
    Button {
      store.send(.tapNext)
    } label: {
      Text(tapPrompt)
        .font(ChangesFont.uiButton)
        .foregroundStyle(ChangesColor.textPrimary)
        .frame(maxWidth: .infinity, minHeight: ChangesSpacing.answerZoneHeight)
        .background(
          RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
            .fill(ChangesColor.surface)
            .overlay(
              RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
                .strokeBorder(
                  store.viewModel?.paused == true
                    ? ChangesColor.Tension.flat13 : ChangesColor.accentBorder)
            )
        )
    }
    .changesAccentGlow()
    .padding(.bottom, 16)
    .accessibilityHint("Advances the session one step and plays audio")
  }
}
