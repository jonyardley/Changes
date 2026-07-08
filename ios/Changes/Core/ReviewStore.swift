import Foundation
import GRDB
import SharedTypes

/// Storage-effect fulfilment seam - a protocol so tests can inject a
/// failing fake.
protocol ReviewStore {
  func load() throws -> [ReviewState]
  func save(state: ReviewState, log: ReviewLog) throws
}

enum ReviewStoreError: Error, CustomStringConvertible {
  case unknownSkillKey(String)
  case storeUnavailable

  var description: String {
    switch self {
    case .unknownSkillKey(let key): "unrecognised skill key in store: \(key)"
    case .storeUnavailable: "local storage is unavailable"
    }
  }
}

/// Last-ditch fallback if even the in-memory database can't open: every op
/// fails loudly, the core surfaces it, the session still runs on an
/// all-new queue.
final class FailingReviewStore: ReviewStore {
  func load() throws -> [ReviewState] { throw ReviewStoreError.storeUnavailable }
  func save(state: ReviewState, log: ReviewLog) throws {
    throw ReviewStoreError.storeUnavailable
  }
}

/// GRDB (SQLite) store executing the core's typed storage effects. Schema
/// is fixed by docs/specs/srs-persistence.md - implement it verbatim,
/// migrations append-only/forward-only (the device is the only copy of the
/// user's practice history).
///
/// Calls are synchronous on purpose: the dataset is tiny (24 skill rows +
/// logs), GRDB's DatabaseQueue serialises access, and an off-main hop
/// isn't worth the Sendable dance against the non-Sendable generated
/// types. Revisit if data volume grows. (Same recorded decision as
/// intrada's LibraryStore.)
final class GrdbReviewStore: ReviewStore {
  private let dbQueue: DatabaseQueue

  init(_ dbQueue: DatabaseQueue) throws {
    self.dbQueue = dbQueue
    try Self.migrator.migrate(dbQueue)
  }

  /// File-backed store in Application Support (the real app).
  static func onDisk() throws -> GrdbReviewStore {
    let dir = try FileManager.default.url(
      for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
    return try GrdbReviewStore(
      DatabaseQueue(path: dir.appendingPathComponent("changes.sqlite").path))
  }

  /// In-memory store for tests and previews.
  static func inMemory() throws -> GrdbReviewStore {
    try GrdbReviewStore(DatabaseQueue())
  }

  // ── ReviewStore ─────────────────────────────────────────────────────

  func load() throws -> [ReviewState] {
    try dbQueue.read { db in
      try Row.fetchAll(
        db,
        sql: "SELECT * FROM review_states WHERE deleted_at IS NULL"
      ).map(Self.state(from:))
    }
  }

  func save(state: ReviewState, log: ReviewLog) throws {
    try dbQueue.write { db in
      // Upsert; updated_at is core-minted time (no shell clock). An upsert
      // revives a tombstoned row.
      try db.execute(
        sql: """
          INSERT INTO review_states
            (skill, stability, difficulty, last_reviewed_at, due_at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, ?, NULL)
          ON CONFLICT(skill) DO UPDATE SET
            stability = excluded.stability,
            difficulty = excluded.difficulty,
            last_reviewed_at = excluded.last_reviewed_at,
            due_at = excluded.due_at,
            updated_at = excluded.updated_at,
            deleted_at = NULL
          """,
        arguments: [
          Self.key(of: state.skill), state.stability, state.difficulty,
          state.lastReviewedAtMs, state.dueAtMs, state.lastReviewedAtMs,
        ])
      try db.execute(
        sql: """
          INSERT INTO review_logs (id, skill, grade, reviewed_at, updated_at, deleted_at)
          VALUES (?, ?, ?, ?, ?, NULL)
          """,
        arguments: [
          log.id, Self.key(of: log.skill), Self.gradeString(log.grade),
          log.reviewedAtMs, log.reviewedAtMs,
        ])
    }
  }

  // ── Codec (spec: skill key "major:3"; grade "got"/"missed") ─────────

  static func key(of skill: SkillId) -> String {
    let mode =
      switch skill.mode {
      case .major: "major"
      case .minor: "minor"
      }
    return "\(mode):\(skill.degree.value)"
  }

  static func skill(from key: String) throws -> SkillId {
    let parts = key.split(separator: ":")
    guard parts.count == 2, let semitones = UInt8(parts[1]), semitones < 12 else {
      throw ReviewStoreError.unknownSkillKey(key)
    }
    let mode: Mode =
      switch parts[0] {
      case "major": .major
      case "minor": .minor
      default: throw ReviewStoreError.unknownSkillKey(key)
      }
    return SkillId(mode: mode, degree: Degree(value: semitones))
  }

  static func gradeString(_ grade: Grade) -> String {
    switch grade {
    case .got: "got"
    case .missed: "missed"
    }
  }

  private static func state(from row: Row) throws -> ReviewState {
    ReviewState(
      skill: try skill(from: row["skill"]),
      stability: row["stability"],
      difficulty: row["difficulty"],
      lastReviewedAtMs: row["last_reviewed_at"],
      dueAtMs: row["due_at"]
    )
  }

  // ── Migrations (append-only, forward-only; vN_description names) ────

  static var migrator: DatabaseMigrator {
    var migrator = DatabaseMigrator()
    migrator.registerMigration("v1_reviews") { db in
      try db.execute(
        sql: """
          CREATE TABLE review_states (
            skill            TEXT PRIMARY KEY,
            stability        REAL NOT NULL,
            difficulty       REAL NOT NULL,
            last_reviewed_at INTEGER NOT NULL,
            due_at           INTEGER NOT NULL,
            updated_at       INTEGER NOT NULL,
            deleted_at       INTEGER
          )
          """)
      try db.execute(
        sql: """
          CREATE TABLE review_logs (
            id          TEXT PRIMARY KEY,
            skill       TEXT NOT NULL,
            grade       TEXT NOT NULL,
            reviewed_at INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL,
            deleted_at  INTEGER
          )
          """)
      try db.execute(
        sql: "CREATE INDEX idx_review_logs_skill ON review_logs(skill, reviewed_at)")
    }
    return migrator
  }
}
