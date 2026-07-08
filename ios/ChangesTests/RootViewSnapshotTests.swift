import SharedTypes
import SnapshotTesting
import SwiftUI
import XCTest

@testable import Changes

/// A bridge serving one canned ViewModel — snapshots render exact states
/// without a core round-trip.
private final class FixtureBridge: CoreBridge {
  let fixture: ViewModel
  init(_ fixture: ViewModel) { self.fixture = fixture }
  func update(_ event: Event) throws -> [Request] { [] }
  func resolve(_ id: UInt32, playScoreOutput: PlayScoreOutput) throws -> [Request] { [] }
  func resolve(_ id: UInt32, storageOutput: StorageOutput) throws -> [Request] { [] }
  func view() throws -> ViewModel { fixture }
}

/// One snapshot per load-bearing Pocket Session state, pinned to the CI
/// simulator (iPhone 16 / iOS 26.5). Rendering differences across runtime
/// versions are real failures — the pin is the contract.
@MainActor
final class RootViewSnapshotTests: XCTestCase {
  private func snapshot(
    _ fixture: ViewModel,
    named name: String,
    sizeCategory: UIContentSizeCategory = .large,
    file: StaticString = #filePath,
    testName: String = #function
  ) throws {
    ChangesFonts.register()
    let store = Store(
      bridge: FixtureBridge(fixture), reviews: try GrdbReviewStore.inMemory())
    let view = RootView()
      .environment(store)
      .environment(\.colorScheme, .dark)
    let controller = UIHostingController(rootView: view)
    controller.overrideUserInterfaceStyle = .dark
    controller.traitOverrides.preferredContentSizeCategory = sizeCategory
    assertSnapshot(
      of: controller,
      as: .image(on: .iPhone13Pro, precision: 0.99, perceptualPrecision: 0.98),
      named: name,
      file: file,
      testName: testName
    )
  }

  private func fixture(
    phase: Phase,
    itemNumber: UInt32 = 3,
    answer: AnswerView? = nil,
    compare: CompareView? = nil,
    recap: RecapView? = nil,
    isPlaying: Bool = false,
    pause: PauseState = .none,
    isLoading: Bool = false,
    error: String? = nil
  ) -> ViewModel {
    ViewModel(
      phase: phase,
      isLoading: isLoading,
      itemNumber: itemNumber,
      totalItems: 12,
      keyName: "E♭",
      answer: answer,
      compare: compare,
      recap: recap,
      isPlaying: isPlaying,
      pause: pause,
      error: error
    )
  }

  func testPreSession() throws {
    try snapshot(fixture(phase: .pre), named: "pre")
  }

  func testContextListening() throws {
    // Bars render at rest in snapshots (animation is runtime-only).
    try snapshot(fixture(phase: .context, isPlaying: true), named: "context")
  }

  func testQuestion() throws {
    try snapshot(fixture(phase: .question, isPlaying: true), named: "question")
  }

  func testGap() throws {
    try snapshot(fixture(phase: .gap), named: "gap")
  }

  func testReveal() throws {
    try snapshot(
      fixture(
        phase: .reveal,
        answer: AnswerView(label: "♭3", resolution: "♭3 · 2 · 1")),
      named: "reveal")
  }

  func testCompare() throws {
    try snapshot(
      fixture(
        phase: .compare,
        answer: AnswerView(label: "♭3", resolution: "♭3 · 2 · 1"),
        compare: CompareView(missed: "♭3", twin: "3", playingTwin: false)),
      named: "compare")
  }

  func testPausedByUser() throws {
    try snapshot(fixture(phase: .gap, pause: .user), named: "paused")
  }

  func testInterrupted() throws {
    try snapshot(fixture(phase: .question, pause: .interrupted), named: "interrupted")
  }

  func testRecap() throws {
    try snapshot(
      fixture(phase: .recap, recap: RecapView(got: 9, missed: 3)),
      named: "recap")
  }

  func testStorageError() throws {
    try snapshot(
      fixture(phase: .context, error: "couldn't access local storage"),
      named: "storage-error")
  }

  // Dynamic Type: the reveal (densest musical state) at accessibility XL.
  func testRevealAtAccessibilityXL() throws {
    try snapshot(
      fixture(
        phase: .reveal,
        answer: AnswerView(label: "♭3", resolution: "♭3 · 2 · 1")),
      named: "reveal-ax-xl",
      sizeCategory: .accessibilityExtraLarge)
  }
}
