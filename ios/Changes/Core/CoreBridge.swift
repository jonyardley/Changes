import SharedTypes

/// Plain-Swift-values seam over the FFI bridge — the only surface a fake
/// replaces for tests and previews.
protocol CoreBridge {
  func update(_ event: Event) throws -> [Request]
  func view() throws -> ViewModel
}
