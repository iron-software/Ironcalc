// IronXL: wasm32 has no JavaScript time-zone database. It supports UTC and the host's local offset.
// The `host` feature selects the same fixed time zones on every target.
#[cfg(any(target_arch = "wasm32", feature = "host"))]
mod fixed_tz;
#[cfg(any(target_arch = "wasm32", feature = "host"))]
pub(crate) use fixed_tz::*;

#[cfg(not(any(target_arch = "wasm32", feature = "host")))]
mod os_tz;
#[cfg(not(any(target_arch = "wasm32", feature = "host")))]
pub(crate) use os_tz::*;
