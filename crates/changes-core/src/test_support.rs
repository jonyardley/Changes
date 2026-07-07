//! Shared test helpers.

use std::fmt::Debug;

use serde::de::DeserializeOwned;
use serde::Serialize;

/// Assert a bridge-crossing type survives the FFI wire format (intrada
/// \#846: a bincode wire break is a silent no-op in the shell, not a crash).
///
/// Uses top-level `bincode::serialize`/`deserialize` — byte-identical to
/// crux 0.19's bridge encoding (fixint, trailing bytes allowed). Don't
/// "upgrade" this to `DefaultOptions` varint; it would no longer match the
/// bridge.
pub(crate) fn assert_bincode_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let bytes = bincode::serialize(value).expect("bincode serialize failed");
    let back: T = bincode::deserialize(&bytes).expect("bincode deserialize failed");
    assert_eq!(value, &back, "bincode round-trip changed the value");
}
