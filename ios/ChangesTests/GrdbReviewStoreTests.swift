import GRDB
import SharedTypes
import XCTest

@testable import Changes

final class GrdbReviewStoreTests: XCTestCase {
  private let now: Int64 = 1_800_000_000_000
  private let dayMs: Int64 = 86_400_000

  private func sampleState(_ semitones: UInt8 = 3, mode: Mode = .major) -> ReviewState {
    ReviewState(
      skill: SkillId(mode: mode, degree: Degree(value: semitones)),
      stability: 2.5,
      difficulty: 5.1,
      lastReviewedAtMs: now,
      dueAtMs: now + 2 * dayMs
    )
  }

  private func sampleLog(_ state: ReviewState, id: String) -> ReviewLog {
    ReviewLog(id: id, skill: state.skill, grade: .got, reviewedAtMs: now)
  }

  func testSaveThenLoadRoundTripsTheState() throws {
    let store = try GrdbReviewStore.inMemory()
    let state = sampleState()

    try store.save(state: state, log: sampleLog(state, id: "01A"))

    XCTAssertEqual(try store.load(), [state])
  }

  func testSaveIsAnUpsertOneRowPerSkill() throws {
    let store = try GrdbReviewStore.inMemory()
    var state = sampleState()
    try store.save(state: state, log: sampleLog(state, id: "01A"))
    state.stability = 9.9
    state.dueAtMs = now + 30 * dayMs

    try store.save(state: state, log: sampleLog(state, id: "01B"))

    let loaded = try store.load()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(loaded.first?.stability, 9.9)
  }

  func testSkillKeyCodecRoundTripsAllTwentyFourSkills() throws {
    for mode in [Mode.major, Mode.minor] {
      for semitones in UInt8(0)..<12 {
        let skill = SkillId(mode: mode, degree: Degree(value: semitones))
        XCTAssertEqual(try GrdbReviewStore.skill(from: GrdbReviewStore.key(of: skill)), skill)
      }
    }
  }

  func testUnknownSkillKeyIsAnErrorNotAGuess() {
    XCTAssertThrowsError(try GrdbReviewStore.skill(from: "lydian:3"))
    XCTAssertThrowsError(try GrdbReviewStore.skill(from: "major:12"))
    XCTAssertThrowsError(try GrdbReviewStore.skill(from: "major"))
  }

  func testSoftDeletedRowsAreFilteredFromLoad() throws {
    let store = try GrdbReviewStore.inMemory()
    let state = sampleState()
    try store.save(state: state, log: sampleLog(state, id: "01A"))

    // No delete op exists yet; simulate a future tombstone directly.
    let queue = try DatabaseQueue()
    try GrdbReviewStore.migrator.migrate(queue)
    try queue.write { db in
      try db.execute(
        sql: """
          INSERT INTO review_states
            (skill, stability, difficulty, last_reviewed_at, due_at, updated_at, deleted_at)
          VALUES ('major:3', 1, 5, 1, 2, 1, 999)
          """)
    }
    let tombstoned = try GrdbReviewStore(queue)
    XCTAssertEqual(try tombstoned.load(), [], "tombstones never load")
  }
}

final class MigrationTests: XCTestCase {
  /// The v1 schema as shipped — frozen SQL, deliberately NOT calling the
  /// current migrator. Future migrations append a fixture here and assert
  /// data written at vN survives migrating to head (CLAUDE.md invariant:
  /// the device is the only copy; a destructive migration is
  /// unrecoverable).
  private func databasePopulatedAtV1() throws -> DatabaseQueue {
    let queue = try DatabaseQueue()
    try queue.write { db in
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
      // Mark v1 as applied so the migrator doesn't re-run it.
      try db.execute(
        sql: "CREATE TABLE grdb_migrations (identifier TEXT NOT NULL PRIMARY KEY)")
      try db.execute(sql: "INSERT INTO grdb_migrations VALUES ('v1_reviews')")
      // Populate.
      try db.execute(
        sql: """
          INSERT INTO review_states VALUES ('major:3', 2.5, 5.1, 100, 200, 100, NULL)
          """)
      try db.execute(
        sql: "INSERT INTO review_logs VALUES ('01A', 'major:3', 'got', 100, 100, NULL)")
    }
    return queue
  }

  func testDatabasePopulatedAtV1MigratesToHeadWithDataIntact() throws {
    let queue = try databasePopulatedAtV1()

    try GrdbReviewStore.migrator.migrate(queue)

    let store = try GrdbReviewStore(queue)
    let loaded = try store.load()
    XCTAssertEqual(loaded.count, 1)
    XCTAssertEqual(loaded.first?.stability, 2.5)
    let logCount = try queue.read { db in
      try Int.fetchOne(db, sql: "SELECT COUNT(*) FROM review_logs") ?? 0
    }
    XCTAssertEqual(logCount, 1)
  }

  func testFreshDatabaseMigratesAndServesTheStore() throws {
    let store = try GrdbReviewStore.inMemory()
    XCTAssertEqual(try store.load(), [])
  }
}
