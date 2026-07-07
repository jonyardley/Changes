import ChangesCoreFFI
import Foundation
import SharedTypes

/// The only place that knows bincode + the UniFFI object. Everything above
/// this speaks typed `SharedTypes` values.
final class LiveBridge: CoreBridge {
  private let core = CoreFfi()

  func update(_ event: Event) throws -> [Request] {
    let out = try core.update(data: Data(try event.bincodeSerialize()))
    return try Requests.bincodeDeserialize(input: [UInt8](out)).value
  }

  func view() throws -> ViewModel {
    try ViewModel.bincodeDeserialize(input: [UInt8](try core.view()))
  }
}
