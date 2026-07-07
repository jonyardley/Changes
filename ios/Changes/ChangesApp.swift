import SwiftUI

@main
struct ChangesApp: App {
  @State private var store = Store()

  init() {
    ChangesFonts.register()
  }

  var body: some Scene {
    WindowGroup {
      RootView()
        .environment(store)
        .preferredColorScheme(.dark)
    }
  }
}
