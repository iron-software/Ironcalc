// IronXL: wasm32 has no JavaScript time-zone database. It supports UTC and the host's local offset.
#[cfg(target_arch = "wasm32")]
mod fixed_tz;
#[cfg(target_arch = "wasm32")]
pub(crate) use fixed_tz::*;

#[cfg(not(target_arch = "wasm32"))]
mod os_tz;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use os_tz::*;
