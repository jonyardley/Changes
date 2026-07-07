import SwiftUI

// Design tokens — 1:1 with design/README.md "Design Tokens (Direction 1a)".
// Spacing, radius, colour, type are tokens, not literals (CLAUDE.md): views
// reach for Changes* namespaces, never hex values or magic numbers.

// ── Colour ──────────────────────────────────────────────────────────────

enum ChangesColor {
  static let background = Color(hex: 0x1714_1C)
  static let backgroundDeep = Color(hex: 0x1411_18)
  static let nearBlack = Color(hex: 0x0E0C_10)

  static let surface = Color(hex: 0x211D_29)
  static let surfaceRaised = Color(hex: 0x2B24_36)

  static let textPrimary = Color(hex: 0xF2EE_F7)
  static let textSecondary = Color(hex: 0x9B93_AB)
  static let textTertiary = Color(hex: 0x6F68_80)
  static let textDisabled = Color(hex: 0x4A44_58)

  static let accent = Color(hex: 0xA58F_E0)
  static let accentBorder = Color(hex: 0xA58F_E0).opacity(0.45)
  static let hairline = Color.white.opacity(0.08)

  /// Chord-quality hues: quality = hue + shape (design "Visual Language" 1).
  enum Quality {
    static let maj7 = Color(hex: 0xD9A8_54)  // gold
    static let m7 = Color(hex: 0x7D99_C9)  // dusty blue
    static let dom7 = Color(hex: 0xCF7A_52)  // coral
    static let m7flat5 = Color(hex: 0xB07F_AE)  // mauve
    static let dim7 = Color(hex: 0x8B8F_A3)  // slate
  }

  /// Tension hues — temperature shifts on the dominant. The system sheet
  /// (canvas 2a/4a) is canonical; the prototype's coral-♭9 is a bug
  /// (mvp-plan decision 2).
  enum Tension {
    static let flat9 = Color(hex: 0xA84C_42)
    static let sharp9 = Color(hex: 0xC25A_8A)
    static let sharp11 = Color(hex: 0x5AA3_A0)
    static let flat13 = Color(hex: 0xB881_3F)
    static let natural13 = Color(hex: 0xD99B_4E)
  }

  /// Mastery = light, never XP: new is a hollow outline in `new`, then the
  /// fill ramps learning → solid → mastered (+ `masteredGlow`).
  enum Mastery {
    static let new = Color(hex: 0x4A44_58)
    static let learning = Color(hex: 0x4A3F_63)
    static let solid = Color(hex: 0x7A68_B3)
    static let mastered = Color(hex: 0xA58F_E0)
  }

  /// Lead-sheet ink (Workbench chart + keyboard diagram keys).
  enum Ink {
    static let cream = Color(hex: 0xECE4_D3)
    static let keyIvory = Color(hex: 0xE9E5_F0)
  }
}

// ── Glow (one glowing object per screen — design rule) ─────────────────

enum ChangesGlow {
  /// Accent glow: `0 0 60px rgba(165,143,224,.28)`.
  static let accentColor = ChangesColor.accent.opacity(0.28)
  static let accentRadius: CGFloat = 30  // shadow radius ≈ half the CSS blur

  /// Mastered-dot glow: `0 0 12px rgba(165,143,224,.8)`.
  static let masteredColor = ChangesColor.accent.opacity(0.8)
  static let masteredRadius: CGFloat = 6
}

extension View {
  /// The at-most-one glow-accented element per screen (play button, current
  /// rung, active card).
  func changesAccentGlow() -> some View {
    shadow(color: ChangesGlow.accentColor, radius: ChangesGlow.accentRadius)
  }

  func changesMasteredGlow() -> some View {
    shadow(color: ChangesGlow.masteredColor, radius: ChangesGlow.masteredRadius)
  }
}

// ── Type ────────────────────────────────────────────────────────────────

/// "The machine speaks grotesque, the music speaks serif" — never mix roles:
/// `ui*` (Space Grotesk) for labels/numbers/buttons, `music*` (Newsreader
/// italic) for chord symbols, keys, quality names, encouragement lines.
/// `relativeTo:` keeps every style tracking Dynamic Type.
enum ChangesFont {
  private enum UI {
    static let regular = "SpaceGrotesk-Regular"
    static let medium = "SpaceGrotesk-Medium"
    static let semibold = "SpaceGrotesk-SemiBold"
    static let bold = "SpaceGrotesk-Bold"
  }

  private enum Music {
    static let italic = "Newsreader-Italic"
    static let mediumItalic = "Newsreader-MediumItalic"
    static let semiboldItalic = "Newsreader-SemiBoldItalic"
  }

  // Machine voice.
  static let uiOverline = Font.custom(UI.medium, size: 13, relativeTo: .caption)
  static let uiBody = Font.custom(UI.regular, size: 15, relativeTo: .body)
  static let uiBodyMedium = Font.custom(UI.medium, size: 15, relativeTo: .body)
  static let uiButton = Font.custom(UI.semibold, size: 16, relativeTo: .body)
  static let uiCounter = Font.custom(UI.medium, size: 14, relativeTo: .footnote)
  static let uiStat = Font.custom(UI.bold, size: 22, relativeTo: .title3)

  /// Overline tracking: 12–14px, .10–.12em uppercase, colour secondary —
  /// apply with `.changesOverline()`.
  static let overlineTracking: CGFloat = 13 * 0.11

  // Music voice.
  static func musicChordSymbol(_ size: CGFloat = 90) -> Font {
    // 84–96px/1.0, −0.02em: tightening is applied at the use site via
    // `.tracking(-size * 0.02)` since Font can't carry it.
    .custom(Music.italic, size: size, relativeTo: .largeTitle)
  }

  static let musicAccentLine = Font.custom(Music.italic, size: 20, relativeTo: .title3)
  static let musicKeyBadge = Font.custom(Music.mediumItalic, size: 16, relativeTo: .callout)
  static func musicHeadline(_ size: CGFloat = 32) -> Font {
    .custom(Music.semiboldItalic, size: size, relativeTo: .title)
  }
}

extension View {
  /// Overline label style: uppercase Space Grotesk, wide tracking, secondary.
  func changesOverline() -> some View {
    font(ChangesFont.uiOverline)
      .textCase(.uppercase)
      .tracking(ChangesFont.overlineTracking)
      .foregroundStyle(ChangesColor.textSecondary)
  }
}

// ── Shape & spacing ─────────────────────────────────────────────────────

enum ChangesSpacing {
  /// Horizontal screen padding.
  static let screenPadding: CGFloat = 28

  static let radiusCard: CGFloat = 20
  static let radiusCardLarge: CGFloat = 22
  static let radiusPill: CGFloat = 99

  /// Answer tap zones: full-width halves, comfortably > 44px min target.
  static let answerZoneHeight: CGFloat = 104

  static let progressBarHeight: CGFloat = 4
  static let progressBarRadius: CGFloat = 2
}

// ── Support ─────────────────────────────────────────────────────────────

extension Color {
  /// Token literals only — views never call this directly.
  fileprivate init(hex: UInt32) {
    self.init(
      .sRGB,
      red: Double((hex >> 16) & 0xFF) / 255,
      green: Double((hex >> 8) & 0xFF) / 255,
      blue: Double(hex & 0xFF) / 255,
      opacity: 1
    )
  }
}
