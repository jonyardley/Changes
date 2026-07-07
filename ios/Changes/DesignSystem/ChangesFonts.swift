import CoreText
import Foundation
import os

/// Registers the bundled Space Grotesk + Newsreader faces with the process
/// font manager (idempotent). The app calls this at launch; snapshot tests
/// will call it too once they land (M3), since they render without the app.
enum ChangesFonts {
  private final class BundleToken {}

  static func register() { _ = didRegister }

  private static let didRegister: Bool = {
    let bundle = Bundle(for: BundleToken.self)
    let faces = [
      "SpaceGrotesk-Regular", "SpaceGrotesk-Medium",
      "SpaceGrotesk-SemiBold", "SpaceGrotesk-Bold",
      "Newsreader-Italic", "Newsreader-MediumItalic",
      "Newsreader-SemiBoldItalic",
    ]
    for face in faces {
      guard let url = bundle.url(forResource: face, withExtension: "ttf") else {
        assertionFailure("Missing bundled font \(face).ttf")
        continue
      }
      var error: Unmanaged<CFError>?
      if !CTFontManagerRegisterFontsForURL(url as CFURL, .process, &error),
        let cfError = error?.takeRetainedValue()
      {
        Logger(subsystem: "com.changes.app", category: "fonts")
          .error(
            "Font registration failed for \(face, privacy: .public): \(cfError.localizedDescription, privacy: .public)"
          )
      }
    }
    return true
  }()
}
