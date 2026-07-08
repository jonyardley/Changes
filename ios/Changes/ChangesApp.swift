import SwiftUI

@main
struct ChangesApp: App {
  @State private var store = Store()

  init() {
    ChangesFonts.register()
  }

  var body: some Scene {
    WindowGroup {
      RootView()
        .environment(store)
        .preferredColorScheme(.dark)
        .task { await autotapIfRequested() }
    }
  }

  /// Debug soak harness, NOT a product mode (the product is manually paced,
  /// mvp-plan decision 9): `--spike-autotap` fires TapNext on a loop so the
  /// M1 exit criteria (5+ minutes of back-to-back playback) can run
  /// unattended on a device, and CI/sim checks can drive the audio path.
  private func autotapIfRequested() async {
    guard ProcessInfo.processInfo.arguments.contains("--spike-autotap") else { return }
    while !Task.isCancelled {
      store.send(.tapNext)
      try? await Task.sleep(nanoseconds: 3_500_000_000)
    }
  }
}
