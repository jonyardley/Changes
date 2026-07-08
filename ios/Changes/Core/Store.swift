import Foundation
import SharedTypes

/// `@Observable` wrapper around the Crux core: renders `ViewModel`, sends
/// `Event`s, fulfils effects. Playback runs on the `ScorePlayer` actor off
/// the main actor and hops back here to resolve.
@MainActor
@Observable
final class Store {
  private(set) var viewModel: ViewModel?
  private(set) var error: String?

  private let bridge: CoreBridge
  private let player = ScorePlayer()
  private let audioSession = AudioSessionMonitor()

  init(bridge: CoreBridge = LiveBridge()) {
    self.bridge = bridge
    self.viewModel = guarded { try bridge.view() }
    audioSession.onInterruption = { [weak self] in self?.pausePlayback(.audioInterrupted) }
    audioSession.onHeadphonesUnplugged = { [weak self] in
      self?.pausePlayback(.headphonesUnplugged)
    }
    audioSession.activate()
  }

  func send(_ event: Event) {
    process(guarded { try bridge.update(event) } ?? [])
  }

  private func pausePlayback(_ event: Event) {
    Task { await player.stop() }
    send(event)
  }

  private func process(_ requests: [Request]) {
    for request in requests {
      switch request.effect {
      case .render:
        refreshView()
      case .playScore(let operation):
        handlePlayScore(operation, id: request.id)
      }
    }
  }

  private func handlePlayScore(_ operation: PlayScoreOperation, id: UInt32) {
    let score = PlayableScore(
      tempoBPM: Double(operation.score.tempoBpm),
      notes: operation.score.notes.map {
        PlayableNote(
          midi: $0.midi,
          onsetBeats: Double($0.onsetBeats),
          durationBeats: Double($0.durationBeats),
          velocity: $0.velocity
        )
      }
    )
    Task { [weak self] in
      var output = PlayScoreOutput.finished
      do {
        try await self?.player.play(score)
      } catch {
        output = .failed(message: String(describing: error))
      }
      self?.resolvePlayback(id, with: output)
    }
  }

  private func resolvePlayback(_ id: UInt32, with output: PlayScoreOutput) {
    process(guarded { try bridge.resolve(id, playScoreOutput: output) } ?? [])
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
