import SharedTypes
import SwiftUI

/// The answer-input picker (docs/specs/answer-input.md decision 1): shown
/// only after the user commits to a recall answer via "I've named it".
/// Chosen 2026-07-08 over strip/arc layout candidates — the arc's labels
/// overlapped illegibly at the 12-option rung 2 case; grid stayed legible
/// and kept generous touch targets at every rung size.
///
/// The tonic carries the screen's one glow (design's "at most one glow per
/// screen" rule): marking "home" ties the picker back to the resolution
/// walk shown at reveal — you're hunting for the way back to this.
struct PickerGrid: View {
  var options: [DegreeOption]
  var onPick: (DegreeOption) -> Void

  private let columns = [GridItem(.adaptive(minimum: 68), spacing: 10)]

  var body: some View {
    LazyVGrid(columns: columns, spacing: 10) {
      ForEach(options, id: \.semitones) { option in
        Button { onPick(option) } label: {
          Text(option.label)
            .font(ChangesFont.musicHeadline(26))
            .foregroundStyle(ChangesColor.textPrimary)
            .frame(maxWidth: .infinity, minHeight: 64)
            .background(
              RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
                .fill(ChangesColor.surface)
                .overlay(
                  RoundedRectangle(cornerRadius: ChangesSpacing.radiusCard)
                    .strokeBorder(
                      option.semitones == 0 ? ChangesColor.accentBorder : ChangesColor.hairline)
                )
            )
        }
        .shadow(
          color: option.semitones == 0 ? ChangesGlow.accentColor : .clear,
          radius: ChangesGlow.accentRadius / 2
        )
        .accessibilityLabel("Degree \(option.label)")
        .accessibilityHint("Submits this as your answer")
      }
    }
  }
}
