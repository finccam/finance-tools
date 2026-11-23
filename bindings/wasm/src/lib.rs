//! `wasm-bindgen` bindings for [`finance_tools`].
//!
//! This crate is a build input of the `finance-tools` npm package
//! (`packages/finance-tools`), which wraps these functions in QuantLib-style
//! classes. It is not published to crates.io.
//!
//! A day counter crosses the boundary as plain values: its QuantLib class
//! name (`"Thirty360"`), its convention's QuantLib name (`"BondBasis"`), the
//! `includeLastDay` flag and an optional termination date. That keeps every
//! function stateless, so JavaScript never has to free wasm memory.
//!
//! Dates cross the boundary as ISO 8601 calendar date strings (`YYYY-MM-DD`).
//! JavaScript `Date` objects are deliberately not accepted: they are instants,
//! and converting them to a civil date depends on a time zone.

// Exported functions take optional strings as `Option<String>`: wasm-bindgen
// cannot pass `Option<&str>` from JavaScript.
#![allow(
    clippy::needless_pass_by_value,
    reason = "wasm-bindgen needs owned optional strings"
)]

use finance_tools::jiff::civil::Date;
use finance_tools::{
    Actual365FixedConvention, ActualActualConvention, DayCounter, Thirty360Convention,
};
use wasm_bindgen::prelude::{JsError, wasm_bindgen};

fn parse_date(field: &str, value: &str) -> Result<Date, JsError> {
    value
        .parse()
        .map_err(|err| JsError::new(&format!("invalid {field} {value:?}: {err}")))
}

fn convention<'a>(kind: &str, convention: Option<&'a str>) -> Result<&'a str, JsError> {
    convention.ok_or_else(|| JsError::new(&format!("{kind} needs a convention")))
}

fn unknown(kind: &str, convention: &str) -> JsError {
    JsError::new(&format!("unknown {kind} convention {convention:?}"))
}

/// Builds the day counter described by QuantLib's class and convention names.
fn day_counter(
    kind: &str,
    convention_name: Option<&str>,
    include_last_day: bool,
    termination_date: Option<&str>,
) -> Result<DayCounter, JsError> {
    Ok(match kind {
        "Actual360" => DayCounter::Actual360 { include_last_day },
        "Actual364" => DayCounter::Actual364,
        "Actual366" => DayCounter::Actual366 { include_last_day },
        "Actual36525" => DayCounter::Actual36525 { include_last_day },
        "Actual365Fixed" => {
            DayCounter::Actual365Fixed(match convention_name.unwrap_or("Standard") {
                "Standard" => Actual365FixedConvention::Standard,
                "Canadian" => Actual365FixedConvention::Canadian,
                "NoLeap" => Actual365FixedConvention::NoLeap,
                other => return Err(unknown(kind, other)),
            })
        }
        "ActualActual" => DayCounter::ActualActual(match convention(kind, convention_name)? {
            "ISMA" => ActualActualConvention::Isma,
            "Bond" => ActualActualConvention::Bond,
            "ISDA" => ActualActualConvention::Isda,
            "Historical" => ActualActualConvention::Historical,
            "Actual365" => ActualActualConvention::Actual365,
            "AFB" => ActualActualConvention::Afb,
            "Euro" => ActualActualConvention::Euro,
            other => return Err(unknown(kind, other)),
        }),
        "Thirty360" => DayCounter::Thirty360 {
            convention: match convention(kind, convention_name)? {
                "USA" => Thirty360Convention::Usa,
                "BondBasis" => Thirty360Convention::BondBasis,
                "European" => Thirty360Convention::European,
                "EurobondBasis" => Thirty360Convention::EurobondBasis,
                "Italian" => Thirty360Convention::Italian,
                "German" => Thirty360Convention::German,
                "ISMA" => Thirty360Convention::Isma,
                "ISDA" => Thirty360Convention::Isda,
                "NASD" => Thirty360Convention::Nasd,
                other => return Err(unknown(kind, other)),
            },
            termination_date: termination_date
                .map(|d| parse_date("termination date", d))
                .transpose()?,
        },
        "Thirty365" => DayCounter::Thirty365,
        "OneDayCounter" => DayCounter::OneDayCounter,
        "SimpleDayCounter" => DayCounter::SimpleDayCounter,
        other => return Err(JsError::new(&format!("unknown day counter {other:?}"))),
    })
}

/// QuantLib's name for a day counter, e.g. `"30/360 (Bond Basis)"`. Also
/// validates the description, so the JavaScript classes call it on
/// construction.
///
/// # Errors
///
/// Throws on an unknown day counter or convention, or an invalid termination
/// date.
#[wasm_bindgen(js_name = dayCounterName)]
pub fn day_counter_name(
    kind: &str,
    convention: Option<String>,
    include_last_day: bool,
    termination_date: Option<String>,
) -> Result<String, JsError> {
    Ok(day_counter(
        kind,
        convention.as_deref(),
        include_last_day,
        termination_date.as_deref(),
    )?
    .name()
    .to_owned())
}

/// `DayCounter::dayCount(d1, d2)`.
///
/// # Errors
///
/// Throws on an invalid day counter description or date.
#[wasm_bindgen(js_name = dayCount)]
#[allow(clippy::cast_precision_loss, reason = "day counts are far below 2^53")]
pub fn day_count(
    kind: &str,
    convention: Option<String>,
    include_last_day: bool,
    termination_date: Option<String>,
    d1: &str,
    d2: &str,
) -> Result<f64, JsError> {
    let dc = day_counter(
        kind,
        convention.as_deref(),
        include_last_day,
        termination_date.as_deref(),
    )?;
    Ok(dc.day_count(parse_date("start date", d1)?, parse_date("end date", d2)?) as f64)
}

/// `DayCounter::yearFraction(d1, d2, refPeriodStart, refPeriodEnd)`. The
/// reference period is optional, but both of its dates must be given or
/// neither.
///
/// # Errors
///
/// Throws on an invalid day counter description or date, on half a reference
/// period, and when the day counter cannot use the reference period (or needs
/// one and none was given).
#[wasm_bindgen(js_name = yearFraction)]
#[allow(clippy::too_many_arguments, reason = "flat on purpose, see crate docs")]
pub fn year_fraction(
    kind: &str,
    convention: Option<String>,
    include_last_day: bool,
    termination_date: Option<String>,
    d1: &str,
    d2: &str,
    ref_period_start: Option<String>,
    ref_period_end: Option<String>,
) -> Result<f64, JsError> {
    let dc = day_counter(
        kind,
        convention.as_deref(),
        include_last_day,
        termination_date.as_deref(),
    )?;
    let (d1, d2) = (parse_date("start date", d1)?, parse_date("end date", d2)?);
    let result = match (ref_period_start, ref_period_end) {
        (None, None) => dc.year_fraction(d1, d2),
        (Some(start), Some(end)) => dc.year_fraction_with_reference_period(
            d1,
            d2,
            parse_date("reference period start", &start)?,
            parse_date("reference period end", &end)?,
        ),
        _ => {
            return Err(JsError::new("give both reference period dates or neither"));
        }
    };
    result.map_err(|err| JsError::new(&format!("{}: {err}", dc.name())))
}

#[cfg(test)]
mod tests {
    use super::*;

    // `JsError` can only be constructed on wasm targets, so only the success
    // paths are tested natively; the JS tests cover the errors.

    #[test]
    fn every_quantlib_name_maps_to_a_day_counter() {
        let cases: &[(&str, &[&str])] = &[
            ("Actual360", &[]),
            ("Actual364", &[]),
            ("Actual366", &[]),
            ("Actual36525", &[]),
            ("Actual365Fixed", &["Standard", "Canadian", "NoLeap"]),
            (
                "ActualActual",
                &[
                    "ISMA",
                    "Bond",
                    "ISDA",
                    "Historical",
                    "Actual365",
                    "AFB",
                    "Euro",
                ],
            ),
            (
                "Thirty360",
                &[
                    "USA",
                    "BondBasis",
                    "European",
                    "EurobondBasis",
                    "Italian",
                    "German",
                    "ISMA",
                    "ISDA",
                    "NASD",
                ],
            ),
            ("Thirty365", &[]),
            ("OneDayCounter", &[]),
            ("SimpleDayCounter", &[]),
        ];
        for (kind, conventions) in cases {
            if conventions.is_empty() {
                assert!(day_counter(kind, None, false, None).is_ok(), "{kind}");
            }
            for convention in *conventions {
                assert!(
                    day_counter(kind, Some(convention), false, None).is_ok(),
                    "{kind} {convention}"
                );
            }
        }
    }

    #[test]
    fn year_fraction_with_and_without_reference_period() {
        let yf = year_fraction(
            "ActualActual",
            Some("ISMA".into()),
            false,
            None,
            "2003-11-01",
            "2004-05-01",
            Some("2003-11-01".into()),
            Some("2004-05-01".into()),
        );
        assert_eq!(yf.ok(), Some(0.5));
        let yf = year_fraction(
            "Actual360",
            None,
            false,
            None,
            "2024-01-01",
            "2024-07-01",
            None,
            None,
        );
        assert_eq!(yf.ok(), Some(182.0 / 360.0));
    }
}
