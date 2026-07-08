import AVFoundation
import Foundation
import os

/// Owns AVAudioSession configuration and turns interruptions / route
/// changes into core events. `.playback` so sessions sound with the
/// ring/silent switch on silent (CLAUDE.md audio trap #1); headphones
/// unplugged must pause, never continue from the speaker.
@MainActor
final class AudioSessionMonitor {
  var onInterruption: (() -> Void)?
  var onHeadphonesUnplugged: (() -> Void)?

  // Owned by the Store for the app's lifetime; observers are never removed
  // (a Swift 6 nonisolated deinit can't touch them anyway).
  private var observers: [NSObjectProtocol] = []
  private let logger = Logger(subsystem: "com.changes.app", category: "audio-session")

  func activate() {
    let session = AVAudioSession.sharedInstance()
    do {
      try session.setCategory(.playback, mode: .default)
      try session.setActive(true)
    } catch {
      // Surfaced on first playback instead: the engine start will fail and
      // resolve the effect as Failed.
      logger.error("audio session activation failed: \(error.localizedDescription, privacy: .public)")
    }

    let center = NotificationCenter.default
    observers.append(
      center.addObserver(
        forName: AVAudioSession.interruptionNotification, object: session, queue: .main
      ) { [weak self] note in
        guard
          let raw = note.userInfo?[AVAudioSessionInterruptionTypeKey] as? UInt,
          AVAudioSession.InterruptionType(rawValue: raw) == .began
        else { return }
        // Manual pacing: we never auto-resume on .ended — the next tap
        // replays the current phase (no item is ever lost).
        MainActor.assumeIsolated { self?.onInterruption?() }
      })
    observers.append(
      center.addObserver(
        forName: AVAudioSession.routeChangeNotification, object: session, queue: .main
      ) { [weak self] note in
        guard
          let raw = note.userInfo?[AVAudioSessionRouteChangeReasonKey] as? UInt,
          AVAudioSession.RouteChangeReason(rawValue: raw) == .oldDeviceUnavailable
        else { return }
        MainActor.assumeIsolated { self?.onHeadphonesUnplugged?() }
      })
  }
}
