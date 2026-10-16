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
    vec![
        "UTC".to_string(),
        "GMT".to_string(),
        "Etc/UTC".to_string(),
        LOCAL.to_string(),
    ]
}

pub(crate) fn excel_serial_for_now(tz: &Tz) -> Option<f64> {
    let offset = match tz {
        Tz::Utc => 0,
        Tz::Local => wasm_host::local_utc_offset_minutes(),
    };
    wasm_host::excel_serial(crate::model::get_milliseconds_since_epoch(), offset)
}

// These tests run with `cargo test -p ironcalc_base --features host`.
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn utc_names_and_local_parse_and_named_zones_do_not() {
        for name in ["UTC", "GMT", "Etc/UTC"] {
            assert!(matches!(Tz::parse(name), Ok(Tz::Utc)), "{name}");
        }
        assert!(matches!(Tz::parse(LOCAL), Ok(Tz::Local)));
        assert_eq!(
            Tz::parse("Europe/Berlin").err().unwrap(),
            "Invalid timezone: Europe/Berlin"
        );
    }

    #[test]
    fn a_model_accepts_the_local_time_zone() {
        assert!(crate::Model::new_empty("model", "en", "Local", "en").is_ok());
        assert!(crate::Model::new_empty("model", "en", "Europe/Berlin", "en").is_err());
    }
}
