import Foundation
import SharedTypes

/// `@Observable` wrapper around the Crux core: renders `ViewModel`, sends
/// `Event`s, fulfils effects. Effect handlers stay off the main actor and
/// hop back to resolve as effect types grow (M1: PlayScore).
@MainActor
@Observable
final class Store {
  private(set) var viewModel: ViewModel?
  private(set) var error: String?

  private let bridge: CoreBridge

  init(bridge: CoreBridge = LiveBridge()) {
    self.bridge = bridge
    self.viewModel = guarded { try bridge.view() }
  }

  func send(_ event: Event) {
    process(guarded { try bridge.update(event) } ?? [])
  }

  private func process(_ requests: [Request]) {
    for request in requests {
      switch request.effect {
      case .render:
        refreshView()
      }
    }
  }

  private func refreshView() {
    if let next = guarded({ try bridge.view() }) {
      viewModel = next
    }
  }

  // Surface, don't swallow: a bridge failure lands in `error`, which the UI
  // must render — a silent bridge no-op is exactly the bug class the
  // round-trip tests exist to catch (intrada #846).
  private func guarded<T>(_ work: () throws -> T) -> T? {
    do {
      let value = try work()
      error = nil
      return value
    } catch {
      self.error = String(describing: error)
      return nil
    }
  }
}
