//! Time zones on wasm32 without JavaScript (IronXL).
//!
//! `"UTC"`, `"GMT"`, and `"Etc/UTC"` use offset zero. `"Local"` uses the host's local UTC offset from
//! `wasm_host::set_local_utc_offset_minutes`. Named time zones such as `"Europe/Berlin"` are not
//! supported.

use crate::wasm_host;

/// The name of the time zone that follows the host's local UTC offset.
pub(crate) const LOCAL: &str = "Local";

#[derive(Clone)]
pub(crate) enum Tz {
    Utc,
    Local,
}

impl Tz {
    pub(crate) fn parse(s: &str) -> Result<Tz, String> {
        match s {
            "UTC" | "GMT" | "Etc/UTC" => Ok(Tz::Utc),
            LOCAL => Ok(Tz::Local),
            _ => Err(format!("Invalid timezone: {s}")),
        }
    }
}

pub fn get_all_timezone_names() -> Vec<String> {
    vec!["UTC".to_string(), "GMT".to_string(), LOCAL.to_string()]
}

pub(crate) fn excel_serial_for_now(tz: &Tz) -> Option<f64> {
    let offset = match tz {
        Tz::Utc => 0,
        Tz::Local => wasm_host::local_utc_offset_minutes(),
    };
    wasm_host::excel_serial(crate::model::get_milliseconds_since_epoch(), offset)
}
