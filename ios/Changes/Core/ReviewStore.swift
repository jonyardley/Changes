import SharedTypes

/// Storage-effect fulfilment seam. The GRDB-backed store (M2b-shell)
/// replaces `InMemoryReviewStore`; the protocol is the contract either way.
protocol ReviewStore {
  func load() throws -> [ReviewState]
  func save(state: ReviewState, log: ReviewLog) throws
}

/// Interim store: correct within a launch (several sessions accumulate
/// reviews), forgotten on relaunch. Honest placeholder until GRDB lands —
/// the core already treats storage as fallible either way.
final class InMemoryReviewStore: ReviewStore {
  private var states: [String: ReviewState] = [:]
  private var logs: [ReviewLog] = []

  func load() throws -> [ReviewState] {
    Array(states.values)
  }

  func save(state: ReviewState, log: ReviewLog) throws {
    states[key(of: state.skill)] = state
    logs.append(log)
  }

  private func key(of skill: SkillId) -> String {
    "\(skill.mode):\(skill.degree)"
  }
}
