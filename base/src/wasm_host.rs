//! Host-supplied clock, local UTC offset, and random seed for `wasm32-unknown-unknown`.
//!
//! IronXL runs the engine in a WebAssembly module without JavaScript. The module has no clock and no
//! source of randomness, so the embedding host sets them before it evaluates formulas:
//!
//! * [`set_milliseconds_since_epoch`]: the current time, read by `NOW()`, `TODAY()`, and conditional
//!   formatting. Set it before each evaluation that can read the clock.
//! * [`set_local_utc_offset_minutes`]: the local UTC offset of the host at that time. The time zone
//!   `"Local"` uses it, so `NOW()` and `TODAY()` match the host's local time.
//! * [`set_random_seed`]: the seed of `RAND()` and `RANDBETWEEN()`.
//!
//! Native builds use this module only with the `host` feature, so an embedder can test natively the
//! code that it runs on wasm32. Without the feature they read the system clock, the system time-zone
//! database, and `rand`, as upstream IronCalc does.

use std::sync::atomic::{AtomicI32, AtomicI64, AtomicU64, Ordering};

use crate::date_system::excel_date_base;

static CLOCK_MS: AtomicI64 = AtomicI64::new(0);
static LOCAL_OFFSET_MINUTES: AtomicI32 = AtomicI32::new(0);
// xorshift64* needs a state that is not zero.
const DEFAULT_SEED: u64 = 0x9E37_79B9_7F4A_7C15;
static RANDOM_STATE: AtomicU64 = AtomicU64::new(DEFAULT_SEED);

/// Sets the current time in milliseconds since 1970-01-01T00:00:00Z.
pub fn set_milliseconds_since_epoch(ms: i64) {
    CLOCK_MS.store(ms, Ordering::Relaxed);
}

/// Sets the local UTC offset of the host in minutes, for example `120` for UTC+02:00.
pub fn set_local_utc_offset_minutes(minutes: i32) {
    LOCAL_OFFSET_MINUTES.store(minutes, Ordering::Relaxed);
}

/// Sets the seed of the random number generator. A seed of zero uses a fixed non-zero seed.
pub fn set_random_seed(seed: u64) {
    RANDOM_STATE.store(non_zero_seed(seed), Ordering::Relaxed);
}

// Tests and the `mock_time` feature use the mock clock instead.
#[cfg_attr(any(test, feature = "mock_time"), allow(dead_code))]
pub(crate) fn milliseconds_since_epoch() -> i64 {
    CLOCK_MS.load(Ordering::Relaxed)
}

pub(crate) fn local_utc_offset_minutes() -> i32 {
    LOCAL_OFFSET_MINUTES.load(Ordering::Relaxed)
}

/// Returns the next random number in `[0, 1)`.
pub(crate) fn random() -> f64 {
    // One atomic step, so two callers never read the same state.
    let previous = RANDOM_STATE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |state| {
            Some(xorshift64_star(state).0)
        })
        .unwrap_or_else(|state| state);
    xorshift64_star(previous).1
}

fn non_zero_seed(seed: u64) -> u64 {
    if seed == 0 {
        DEFAULT_SEED
    } else {
        seed
    }
}

/// One step of xorshift64*: the next state and a number in `[0, 1)` with 53 random bits.
fn xorshift64_star(state: u64) -> (u64, f64) {
    let mut x = state;
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    let value = (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64;
    (x, value)
}

/// The Excel serial number of `ms` (milliseconds since the Unix epoch) at a fixed UTC offset.
pub(crate) fn excel_serial(ms: i64, offset_minutes: i32) -> Option<f64> {
    use chrono::{DateTime, Datelike, Timelike};
    // Whole seconds, as the native path does.
    let local_seconds = ms
        .div_euclid(1000)
        .checked_add(i64::from(offset_minutes) * 60)?;
    let dt = DateTime::from_timestamp(local_seconds, 0)?;
    let days = dt.num_days_from_ce() - excel_date_base();
    Some(f64::from(days) + f64::from(dt.num_seconds_from_midnight()) / 86400.0)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn excel_serial_of_the_unix_epoch_is_25569() {
        assert_eq!(excel_serial(0, 0), Some(25569.0));
    }

    #[test]
    fn excel_serial_uses_the_date_system_of_the_model() {
        let _dates = crate::date_system::DateSystemScope::enter(true);
        assert_eq!(excel_serial(0, 0), Some(25569.0 - 1462.0));
    }

    #[test]
    fn excel_serial_applies_the_local_offset() {
        // 2026-10-08T23:30:00Z is 2026-10-09T01:30 at UTC+02:00, the next day.
        let ms = 1_791_502_200_000;
        let utc = excel_serial(ms, 0).unwrap();
        let local = excel_serial(ms, 120).unwrap();
        assert_eq!(utc.floor() + 1.0, local.floor());
        assert!((local - utc - 120.0 / 1440.0).abs() < 1e-9);
        assert!((excel_serial(ms, -300).unwrap() - (utc - 300.0 / 1440.0)).abs() < 1e-9);
    }

    #[test]
    fn excel_serial_rejects_an_out_of_range_time() {
        assert_eq!(excel_serial(i64::MAX, 0), None);
        assert_eq!(excel_serial(i64::MAX - 1, i32::MAX), None);
    }

    #[test]
    fn xorshift_gives_numbers_in_the_unit_interval_and_a_new_state() {
        let mut state = DEFAULT_SEED;
        for _ in 0..1000 {
            let (next, value) = xorshift64_star(state);
            assert!((0.0..1.0).contains(&value));
            assert_ne!(next, state);
            state = next;
        }
    }

    #[test]
    fn a_zero_seed_falls_back_to_the_default_seed() {
        assert_eq!(non_zero_seed(0), DEFAULT_SEED);
        assert_eq!(non_zero_seed(7), 7);
    }

    #[test]
    fn the_setters_store_the_host_values() {
        set_milliseconds_since_epoch(1234);
        assert_eq!(milliseconds_since_epoch(), 1234);
        set_local_utc_offset_minutes(-90);
        assert_eq!(local_utc_offset_minutes(), -90);
        set_random_seed(42);
        let first = random();
        set_random_seed(42);
        assert_eq!(random(), first);
    }
}
