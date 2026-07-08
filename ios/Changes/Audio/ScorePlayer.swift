import AVFoundation
import Foundation

/// A score note in plain Sendable values, converted from the generated
/// `SharedTypes.Score` at the Store boundary so no generated type has to
/// cross into the audio actor.
struct PlayableNote: Sendable {
  let midi: UInt8
  let onsetBeats: Double
  let durationBeats: Double
  let velocity: UInt8
}

struct PlayableScore: Sendable {
  let tempoBPM: Double
  let notes: [PlayableNote]

  var durationSeconds: Double {
    let endBeats = notes.map { $0.onsetBeats + $0.durationBeats }.max() ?? 0
    return endBeats * 60.0 / tempoBPM
  }
}

enum ScorePlayerError: Error, CustomStringConvertible {
  case soundFontMissing
  case engineStart(String)
  case sequencerStart(String)

  var description: String {
    switch self {
    case .soundFontMissing: "bundled SoundFont not found"
    case .engineStart(let detail): "audio engine failed to start: \(detail)"
    case .sequencerStart(let detail): "sequencer failed to start: \(detail)"
    }
  }
}

/// Realizes core-emitted scores on AVAudioEngine + SoundFont sampler.
/// Notes are pre-scheduled on an `AVAudioSequencer` (sample-accurate by
/// construction), not per-note dispatched (mvp-plan M1 risk note). The
/// shell never invents or re-voices content — scores play verbatim.
actor ScorePlayer {
  private let engine = AVAudioEngine()
  private let sampler = AVAudioUnitSampler()
  private var sequencer: AVAudioSequencer?
  private var prepared = false

  /// Plays the score and returns when playback has finished. Throws on real
  /// failures (missing soundfont, engine/sequencer start) — the caller
  /// resolves those as `PlayScoreOutput.failed`, never a silent success.
  func play(_ score: PlayableScore) async throws {
    try prepareIfNeeded()

    if !engine.isRunning {
      do {
        try engine.start()
      } catch {
        throw ScorePlayerError.engineStart(error.localizedDescription)
      }
    }

    stopSequencer()

    let sequencer = AVAudioSequencer(audioEngine: engine)
    let track = sequencer.createAndAppendTrack()
    track.destinationAudioUnit = sampler
    sequencer.tempoTrack.addEvent(AVExtendedTempoEvent(tempo: score.tempoBPM), at: 0)
    for note in score.notes {
      let event = AVMIDINoteEvent(
        channel: 0,
        key: UInt32(note.midi),
        velocity: UInt32(note.velocity),
        duration: AVMusicTimeStamp(note.durationBeats)
      )
      track.addEvent(event, at: AVMusicTimeStamp(note.onsetBeats))
    }

    sequencer.prepareToPlay()
    do {
      try sequencer.start()
    } catch {
      throw ScorePlayerError.sequencerStart(error.localizedDescription)
    }
    self.sequencer = sequencer

    // The sequencer owns sample-accurate timing; this sleep only decides
    // when we report completion (+ a small release tail).
    try? await Task.sleep(nanoseconds: UInt64((score.durationSeconds + 0.25) * 1_000_000_000))
    stopSequencer()
  }

  /// Interruption/route-change hard stop. Idempotent.
  func stop() {
    stopSequencer()
    engine.pause()
  }

  private func stopSequencer() {
    sequencer?.stop()
    sequencer = nil
  }

  private func prepareIfNeeded() throws {
    guard !prepared else { return }
    guard
      let url = Bundle.main.url(forResource: "GeneralUser-GS", withExtension: "sf2")
    else {
      throw ScorePlayerError.soundFontMissing
    }
    engine.attach(sampler)
    engine.connect(sampler, to: engine.mainMixerNode, format: nil)
    engine.prepare()
    // Acoustic grand (GM program 0) from the melodic bank.
    try sampler.loadSoundBankInstrument(
      at: url,
      program: 0,
      bankMSB: UInt8(kAUSampler_DefaultMelodicBankMSB),
      bankLSB: UInt8(kAUSampler_DefaultBankLSB)
    )
    prepared = true
  }
}
