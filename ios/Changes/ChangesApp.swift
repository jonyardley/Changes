import SharedTypes
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

  /// The soak can't know the right answer (that's the point of the app);
  /// it picks the first option, which is right ~1/7th of the time — both
  /// paths get exercised either way.
  private func correctish(_ vm: ViewModel) -> DegreeOption {
    vm.options.first ?? DegreeOption(label: "1", semitones: 0)
  }

  /// Debug soak harness, NOT a product mode (the product is manually paced,
  /// mvp-plan decision 9): `--spike-autotap` walks whole sessions unattended
  /// (misses every 4th item to exercise the compare loop) so the M1 device
  /// soak and sim checks can drive the audio path hands-free.
  private func autotapIfRequested() async {
    guard ProcessInfo.processInfo.arguments.contains("--spike-autotap") else { return }
    while !Task.isCancelled {
      switch store.viewModel?.phase {
      case .pre, .recap:
        let nowMs = Int64(Date.now.timeIntervalSince1970 * 1000)
        store.send(.startSession(seed: 2_026_07_08, nowMs: nowMs, maxItems: 12))
      case .gap:
        store.send(.tapReady)
      case .pick:
        // Answer wrongly every 4th item to exercise the compare loop.
        if let vm = store.viewModel, let first = vm.options.first {
          let wrongOption = vm.options.last ?? first
          let pick = vm.itemNumber % 4 == 0 ? wrongOption : correctish(vm)
          store.send(.submitAnswer(degree: Degree(value: pick.semitones)))
        }
      case .reveal:
        store.send(.tapNext)
      case .compare:
        store.send(.exitCompare)
      case .context, .question, .none:
        break
      }
      try? await Task.sleep(nanoseconds: 2_500_000_000)
    }
  }
}
