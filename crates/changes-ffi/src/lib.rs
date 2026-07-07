//! FFI bridge crate: uniffi scaffolding for the iOS shell plus the facet
//! typegen driver (`bin/codegen.rs`) that emits the Swift `SharedTypes`
//! package.

pub use changes_core::*;

pub mod ffi;
pub use ffi::{CoreError, CoreFFI};

// Compile-time version assertion — uniffi skew vs cargo-swift's bindgen is a
// compile error, not a runtime crash.
#[cfg(feature = "uniffi")]
const _: () = assert!(
    uniffi::check_compatible_version("0.29.4"),
    "use uniffi v0.29.4 to match cargo-swift 0.9.0's bindgen"
);
#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
