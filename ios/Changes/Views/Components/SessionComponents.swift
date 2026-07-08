import SwiftUI

// Session building blocks, pixel-close to design canvases 1a/2b/4b
// (design/README.md "Screens / Views", Journey 1).

// ── Level bars (context/listening: no pitch visuals, level only) ────────

struct LevelBars: View {
  var animating: Bool
  @State private var grown = false

  var body: some View {
    HStack(spacing: 6) {
      ForEach(0..<7, id: \.self) { index in
        RoundedRectangle(cornerRadius: 2)
          .fill(ChangesColor.accent)
          .frame(width: 6, height: 44)
          .scaleEffect(y: grown ? 1.0 : 0.3, anchor: .center)
          .animation(
            animating
              ? .easeInOut(duration: 0.55 + Double(index) * 0.07)
                .repeatForever(autoreverses: true)
              : .default,
            value: grown
          )
      }
    }
    .onAppear { grown = animating }
    .onChange(of: animating) { _, now in grown = now }
    .accessibilityHidden(true)
  }
}

// ── Question pulse ("?" in a 150px circle, soft 1.6s pulse) ─────────────

struct QuestionPulse: View {
  @State private var dimmed = false

  var body: some View {
    ZStack {
      Circle()
        .strokeBorder(ChangesColor.accentBorder, lineWidth: 1.5)
        .frame(width: 150, height: 150)
      Text("?")
        .font(ChangesFont.musicHeadline(64))
        .foregroundStyle(ChangesColor.accent)
        .opacity(dimmed ? 0.55 : 1.0)
        .animation(
          .easeInOut(duration: 1.6).repeatForever(autoreverses: true), value: dimmed)
    }
    .onAppear { dimmed = true }
    .accessibilityHidden(true)
  }
}

// ── Big circular play button (pre-session, 190px, the one glow) ─────────

struct PlayButton: View {
  var label: String
  var enabled: Bool
  var action: () -> Void

  var body: some View {
    Button(action: action) {
      ZStack {
        Circle()
          .fill(
            RadialGradient(
              colors: [ChangesColor.surfaceRaised, ChangesColor.surface],
              center: .center, startRadius: 10, endRadius: 95)
          )
          .overlay(Circle().strokeBorder(ChangesColor.accentBorder, lineWidth: 1.5))
        Text(label)
          .font(ChangesFont.uiButton)
          .foregroundStyle(ChangesColor.textPrimary)
      }
      .frame(width: 190, height: 190)
    }
    .changesAccentGlow()
    .disabled(!enabled)
    .accessibilityLabel(label)
    .accessibilityHint("Starts today's session and plays audio")
  }
}

// ── Duration pills (10 / 18 / 25 minutes) ───────────────────────────────

struct DurationPills: View {
  @Binding var minutes: Int
  private let options = [10, 18, 25]

  var body: some View {
    HStack(spacing: 10) {
      ForEach(options, id: \.self) { option in
        Button {
          minutes = option
        } label: {
          Text("\(option)")
            .font(ChangesFont.uiBodyMedium)
            .foregroundStyle(
              minutes == option ? ChangesColor.textPrimary : ChangesColor.textTertiary
            )
            .padding(.horizontal, 18)
            .padding(.vertical, 8)
            .background(
              Capsule()
                .fill(minutes == option ? ChangesColor.surfaceRaised : ChangesColor.surface)
                .overlay(
                  Capsule().strokeBorder(
                    minutes == option ? ChangesColor.accentBorder : ChangesColor.hairline)
                )
            )
        }
        .accessibilityLabel("\(option) minute session")
        .accessibilityAddTraits(minutes == option ? .isSelected : [])
      }
      Text("min")
        .changesOverline()
    }
  }
}

// ── Recap ledger row ─────────────────────────────────────────────────────

struct LedgerRow: View {
  var label: String
  var value: String
  var glow: Bool = false

  var body: some View {
    HStack {
      Text(label)
        .font(ChangesFont.uiBody)
        .foregroundStyle(ChangesColor.textSecondary)
      Spacer()
      Text(value)
        .font(ChangesFont.uiBodyMedium)
        .foregroundStyle(glow ? ChangesColor.Mastery.mastered : ChangesColor.textPrimary)
    }
    .padding(.vertical, 10)
    .overlay(alignment: .bottom) {
      Rectangle().fill(ChangesColor.hairline).frame(height: 1)
    }
    .accessibilityElement(children: .combine)
    .accessibilityLabel("\(label): \(value)")
  }
}
