import SharedTypes
import SwiftUI

/// M0 walking skeleton: proves Event::Ping → core → ViewModel through the
/// real bridge, wearing the design tokens. Replaced by the Pocket Session
/// surfaces from M3.
struct RootView: View {
  @Environment(Store.self) private var store

  var body: some View {
    VStack(spacing: 32) {
      Spacer()

      Text("Changes")
        .changesOverline()
        .accessibilityAddTraits(.isHeader)

      Text("learn to hear changes")
        .font(ChangesFont.musicAccentLine)
        .foregroundStyle(ChangesColor.accent)

      Button {
        store.send(.ping)
      } label: {
        Text("Ping the core")
          .font(ChangesFont.uiButton)
          .foregroundStyle(ChangesColor.textPrimary)
          .padding(.horizontal, 28)
          .padding(.vertical, 14)
          .background(
            Capsule()
              .fill(ChangesColor.surface)
              .overlay(Capsule().strokeBorder(ChangesColor.accentBorder))
          )
      }
      .changesAccentGlow()
      .accessibilityHint("Sends a test event through the Rust core")

      Text("pongs: \(store.viewModel?.pongCount ?? 0)")
        .font(ChangesFont.uiCounter)
        .foregroundStyle(ChangesColor.textSecondary)
        .accessibilityLabel("Pong count \(store.viewModel?.pongCount ?? 0)")

      if let error = store.error {
        Text(error)
          .font(ChangesFont.uiBody)
          .foregroundStyle(ChangesColor.Tension.flat9)
      }

      Spacer()
    }
    .frame(maxWidth: .infinity, maxHeight: .infinity)
    .padding(.horizontal, ChangesSpacing.screenPadding)
    .background(ChangesColor.background.ignoresSafeArea())
  }
}
