//! Day count conventions.
//!
//! A day count convention determines how interest accrues over time: it maps a
//! pair of dates to a *year fraction* (also called the day count factor).
//!
//! The conventions and their exact semantics mirror
//! [`RustQuant_time` 0.4](https://docs.rs/RustQuant_time/0.4.0) so that results
//! are interchangeable. Parity is enforced by the tests of the unpublished
//! `finance-tools-parity` crate in this crate's repository. Where `RustQuant`
//! deviates from the ISDA / ICMA definitions, the variant documentation says
//! so.
//!
//! Act/Act ICMA is intentionally absent: `RustQuant` declares it but does not
//! implement it (`todo!()`), and a correct implementation needs coupon
//! schedule information that a two-date API cannot carry.

use core::fmt;

use jiff::civil::Date;

const SECONDS_PER_DAY: i64 = 86_400;

/// A day count convention.
///
/// Use [`DayCountConvention::year_fraction`] to compute the year fraction
/// between two dates. Dates are not required to be ordered; a reversed pair
/// yields the value the underlying formula produces (negative for the
/// actual-day based conventions).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DayCountConvention {
    /// `1/1`: always `1.0`, regardless of the dates.
    OneOne,
    /// `Act/360`: actual days divided by 360.
    Actual360,
    /// `Act/364`: actual days divided by 364.
    Actual364,
    /// `Act/366`: actual days divided by 366.
    Actual366,
    /// `Act/365.25`: actual days divided by 365.25.
    Actual365_25,
    /// `Act/365 Actual`: actual days divided by 366 if any calendar year
    /// touched by the period (start year through end year, inclusive) is a
    /// leap year, otherwise by 365.
    ///
    /// Note: this checks for leap *years* in range, not for a 29 February
    /// falling inside the period (`RustQuant` semantics).
    Actual365Actual,
    /// `Act/365F`: actual days divided by 365.
    Actual365Fixed,
    /// `Act/365L`: computed identically to [`Self::Actual365Actual`].
    ///
    /// Note: `RustQuant` does not implement the ISDA/ICMA definition, which
    /// depends on the end date's year and the coupon frequency.
    Actual365Leap,
    /// `Act/Act AFB`: whole years counted back from the end date, plus the
    /// initial stub divided by 366 if the stub touches a leap year; otherwise
    /// actual days divided by 365.
    ///
    /// When the end date is 29 February and the stub year has no such day,
    /// 28 February is used as the stub anchor (`RustQuant` panics here).
    ActualActualAfb,
    /// `Act/Act ISDA`: days falling in leap years are divided by 366, days in
    /// other years by 365.
    ActualActualIsda,
    /// `NL/360`: actual days minus the number of leap years touched by the
    /// period, divided by 360.
    ///
    /// Note: this subtracts one day per leap *year* in range, whether or not
    /// 29 February falls inside the period (`RustQuant` semantics).
    NoLeap360,
    /// `NL/365`: as [`Self::NoLeap360`], divided by 365.
    NoLeap365,
    /// `30/360 ISDA` (bond basis): start day 31 becomes 30; end day 31 becomes
    /// 30 if the (adjusted) start day is 30.
    Thirty360Isda,
    /// `30E/360` (Eurobond basis): day 31 becomes 30 on both dates.
    ThirtyE360,
    /// `30E/360 ISDA`: day 31 or the last day of February becomes 30 on both
    /// dates.
    ///
    /// Note: unlike the ISDA 2006 definition, the end date's February rule is
    /// applied even when it is the maturity date (`RustQuant` semantics).
    ThirtyE360Isda,
    /// `30E/365`: as [`Self::ThirtyE360Isda`], divided by 365.
    ThirtyE365,
    /// `30E+/360`: start day 31 becomes 30; end day 31 rolls to day 1 of the
    /// following month.
    ThirtyEPlus360,
    /// `30U/360` (US): start day 31 or last day of February becomes 30; end
    /// day becomes 30 if it is 31 and the adjusted start day is 30, or if it is
    /// the last day of February.
    ///
    /// Note: the end date's February rule is applied without requiring the
    /// start date to also be the last day of February (`RustQuant` semantics).
    ThirtyU360,
}

impl DayCountConvention {
    /// Every supported convention, in declaration order.
    pub const ALL: [Self; 18] = [
        Self::OneOne,
        Self::Actual360,
        Self::Actual364,
        Self::Actual366,
        Self::Actual365_25,
        Self::Actual365Actual,
        Self::Actual365Fixed,
        Self::Actual365Leap,
        Self::ActualActualAfb,
        Self::ActualActualIsda,
        Self::NoLeap360,
        Self::NoLeap365,
        Self::Thirty360Isda,
        Self::ThirtyE360,
        Self::ThirtyE360Isda,
        Self::ThirtyE365,
        Self::ThirtyEPlus360,
        Self::ThirtyU360,
    ];

    /// The conventional market name, e.g. `"Act/365F"` or `"30E/360 ISDA"`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OneOne => "1/1",
            Self::Actual360 => "Act/360",
            Self::Actual364 => "Act/364",
            Self::Actual366 => "Act/366",
            Self::Actual365_25 => "Act/365.25",
            Self::Actual365Actual => "Act/365 Actual",
            Self::Actual365Fixed => "Act/365F",
            Self::Actual365Leap => "Act/365L",
            Self::ActualActualAfb => "Act/Act AFB",
            Self::ActualActualIsda => "Act/Act ISDA",
            Self::NoLeap360 => "NL/360",
            Self::NoLeap365 => "NL/365",
            Self::Thirty360Isda => "30/360 ISDA",
            Self::ThirtyE360 => "30E/360",
            Self::ThirtyE360Isda => "30E/360 ISDA",
            Self::ThirtyE365 => "30E/365",
            Self::ThirtyEPlus360 => "30E+/360",
            Self::ThirtyU360 => "30U/360",
        }
    }

    /// Computes the year fraction between `start` and `end` under this
    /// convention.
    ///
    /// ```
    /// use finance_tools::DayCountConvention;
    /// use jiff::civil::date;
    ///
    /// let yf = DayCountConvention::ActualActualIsda.year_fraction(date(2003, 11, 1), date(2004, 5, 1));
    /// assert!((yf - 0.497_724_380_567).abs() < 1e-12);
    /// ```
    #[must_use]
    pub fn year_fraction(self, start: Date, end: Date) -> f64 {
        match self {
            Self::OneOne => 1.0,
            Self::Actual360 => actual_days(start, end) / 360.0,
            Self::Actual364 => actual_days(start, end) / 364.0,
            Self::Actual366 => actual_days(start, end) / 366.0,
            Self::Actual365_25 => actual_days(start, end) / 365.25,
            Self::Actual365Fixed => actual_days(start, end) / 365.0,
            Self::Actual365Actual | Self::Actual365Leap => {
                let basis = if any_leap_year(start.year().into(), end.year().into()) {
                    366.0
                } else {
                    365.0
                };
                actual_days(start, end) / basis
            }
            Self::ActualActualAfb => actual_actual_afb(start, end),
            Self::ActualActualIsda => actual_actual_isda(start, end),
            Self::NoLeap360 => no_leap_days(start, end) / 360.0,
            Self::NoLeap365 => no_leap_days(start, end) / 365.0,
            Self::Thirty360Isda => thirty_360_isda(start, end),
            Self::ThirtyE360 => thirty_e_360(start, end),
            Self::ThirtyE360Isda => thirty_e_isda_days(start, end) / 360.0,
            Self::ThirtyE365 => thirty_e_isda_days(start, end) / 365.0,
            Self::ThirtyEPlus360 => thirty_e_plus_360(start, end),
            Self::ThirtyU360 => thirty_u_360(start, end),
        }
    }
}

impl fmt::Display for DayCountConvention {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

// ---------------------------------------------------------------------------
// Calendar helpers
// ---------------------------------------------------------------------------

/// Proleptic Gregorian leap year rule, on `i32` so callers may step past
/// `jiff`'s supported year range without overflow.
const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Number of leap years in `first..=last` (zero if `first > last`).
fn leap_years_in(first: i32, last: i32) -> i32 {
    let leaps_up_to = |y: i32| y.div_euclid(4) - y.div_euclid(100) + y.div_euclid(400);
    if first > last {
        0
    } else {
        leaps_up_to(last) - leaps_up_to(first - 1)
    }
}

fn any_leap_year(first: i32, last: i32) -> bool {
    leap_years_in(first, last) > 0
}

/// Signed number of calendar days from `start` to `end`.
fn days_between(start: Date, end: Date) -> i32 {
    let days = end.duration_since(start).as_secs() / SECONDS_PER_DAY;
    // `jiff` dates span years -9999..=9999, i.e. fewer than 7.4 million days.
    i32::try_from(days).expect("jiff's date range fits in i32 days")
}

fn actual_days(start: Date, end: Date) -> f64 {
    f64::from(days_between(start, end))
}

/// 1-based day of the year for a month/day in a year of the given leapness.
fn ordinal(leap: bool, month: i8, day: i8) -> i32 {
    const DAYS_BEFORE_MONTH: [i32; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let month_index = usize::try_from(month - 1).expect("jiff months are 1..=12");
    let leap_day = i32::from(leap && month > 2);
    DAYS_BEFORE_MONTH[month_index] + leap_day + i32::from(day)
}

fn is_last_day_of_february(date: Date) -> bool {
    date.month() == 2 && date.day() == date.days_in_month()
}

// ---------------------------------------------------------------------------
// Actual/Actual
// ---------------------------------------------------------------------------

fn days_in_year(leap: bool) -> f64 {
    if leap { 366.0 } else { 365.0 }
}

fn actual_actual_isda(start: Date, end: Date) -> f64 {
    if start == end {
        return 0.0;
    }

    let (y1, y2) = (i32::from(start.year()), i32::from(end.year()));

    // Days from `start` up to 1 January of the following year, and from
    // 1 January of the end year up to `end`. Computed from ordinals so that
    // year 9999 does not require constructing an out-of-range date.
    let days_to_year_end = i32::from(start.days_in_year() - start.day_of_year() + 1);
    let days_from_year_start = i32::from(end.day_of_year() - 1);

    let mut sum = f64::from(y2 - y1 - 1);
    sum += f64::from(days_to_year_end) / days_in_year(start.in_leap_year());
    sum += f64::from(days_from_year_start) / days_in_year(end.in_leap_year());
    sum
}

fn actual_actual_afb(start: Date, end: Date) -> f64 {
    let y1 = i32::from(start.year());
    let (m2, d2) = (end.month(), end.day());

    // Anchor the stub on the end date's month/day, in the start year if that
    // is not before `start`, otherwise in the following year.
    let anchor_day = |year: i32| {
        if m2 == 2 && d2 == 29 && !is_leap_year(year) {
            28
        } else {
            d2
        }
    };
    let stub_in_start_year = (m2, anchor_day(y1)) >= (start.month(), start.day());
    let stub_year = if stub_in_start_year { y1 } else { y1 + 1 };
    let stub_day = anchor_day(stub_year);

    let initial_stub_days = if stub_in_start_year {
        ordinal(is_leap_year(y1), m2, stub_day) - i32::from(start.day_of_year())
    } else {
        i32::from(start.days_in_year() - start.day_of_year())
            + ordinal(is_leap_year(stub_year), m2, stub_day)
    };

    if any_leap_year(y1, stub_year) {
        let final_stub_years = i32::from(end.year()) - stub_year;
        f64::from(final_stub_years) + f64::from(initial_stub_days) / 366.0
    } else {
        actual_days(start, end) / 365.0
    }
}

// ---------------------------------------------------------------------------
// No-leap
// ---------------------------------------------------------------------------

fn no_leap_days(start: Date, end: Date) -> f64 {
    let leap_years = leap_years_in(start.year().into(), end.year().into());
    f64::from(days_between(start, end) - leap_years)
}

// ---------------------------------------------------------------------------
// 30/360 family
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Ymd {
    year: i32,
    month: i32,
    day: i32,
}

impl From<Date> for Ymd {
    fn from(date: Date) -> Self {
        Self {
            year: date.year().into(),
            month: date.month().into(),
            day: date.day().into(),
        }
    }
}

fn thirty_360_days(start: Ymd, end: Ymd) -> f64 {
    f64::from(
        360 * (end.year - start.year) + 30 * (end.month - start.month) + (end.day - start.day),
    )
}

fn thirty_360_isda(start: Date, end: Date) -> f64 {
    let (mut s, mut e) = (Ymd::from(start), Ymd::from(end));
    if s.day == 31 {
        s.day = 30;
    }
    if s.day == 30 && e.day == 31 {
        e.day = 30;
    }
    thirty_360_days(s, e) / 360.0
}

fn thirty_e_360(start: Date, end: Date) -> f64 {
    let (mut s, mut e) = (Ymd::from(start), Ymd::from(end));
    if s.day == 31 {
        s.day = 30;
    }
    if e.day == 31 {
        e.day = 30;
    }
    thirty_360_days(s, e) / 360.0
}

fn thirty_e_isda_days(start: Date, end: Date) -> f64 {
    let (mut s, mut e) = (Ymd::from(start), Ymd::from(end));
    if s.day == 31 || is_last_day_of_february(start) {
        s.day = 30;
    }
    if e.day == 31 || is_last_day_of_february(end) {
        e.day = 30;
    }
    thirty_360_days(s, e)
}

fn thirty_e_plus_360(start: Date, end: Date) -> f64 {
    let (mut s, mut e) = (Ymd::from(start), Ymd::from(end));
    if s.day == 31 {
        s.day = 30;
    }
    if e.day == 31 {
        // Roll to the first of the next month. Done arithmetically so that
        // 9999-12-31 does not overflow `jiff`'s date range.
        e = if e.month == 12 {
            Ymd {
                year: e.year + 1,
                month: 1,
                day: 1,
            }
        } else {
            Ymd {
                month: e.month + 1,
                day: 1,
                ..e
            }
        };
    }
    thirty_360_days(s, e) / 360.0
}

fn thirty_u_360(start: Date, end: Date) -> f64 {
    let (mut s, mut e) = (Ymd::from(start), Ymd::from(end));
    if s.day == 31 || is_last_day_of_february(start) {
        s.day = 30;
    }
    if (e.day == 31 && s.day == 30) || is_last_day_of_february(end) {
        e.day = 30;
    }
    thirty_360_days(s, e) / 360.0
}

#[cfg(test)]
mod tests {
    use jiff::civil::date;

    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-12,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn leap_year_counting_matches_naive() {
        for first in -500..=500 {
            for last in [first - 1, first, first + 3, first + 100, first + 401] {
                let naive = (first..=last).filter(|&y| is_leap_year(y)).count();
                assert_eq!(
                    usize::try_from(leap_years_in(first, last)).unwrap(),
                    naive,
                    "{first}..={last}"
                );
            }
        }
    }

    #[test]
    fn ordinal_matches_jiff() {
        for year in [2023_i16, 2024] {
            let mut d = date(year, 1, 1);
            while d.year() == year {
                assert_eq!(
                    ordinal(d.in_leap_year(), d.month(), d.day()),
                    i32::from(d.day_of_year())
                );
                d = d.tomorrow().unwrap();
            }
        }
    }

    // Reference values from QuantLib, as used in `RustQuant`'s own test suite.
    #[test]
    fn quantlib_reference_values() {
        let (d1, d2) = (date(2003, 11, 1), date(2004, 5, 1));
        assert_close(
            DayCountConvention::ActualActualIsda.year_fraction(d1, d2),
            0.497_724_380_567,
        );
        assert_close(
            DayCountConvention::ActualActualAfb.year_fraction(d1, d2),
            0.497_267_759_563,
        );
        assert_close(
            DayCountConvention::ThirtyE365.year_fraction(date(2011, 6, 17), date(2012, 12, 30)),
            1.515_068_493_150_685,
        );
    }

    #[test]
    fn afb_leap_day_end_falls_back_to_feb_28() {
        // Stub anchor 2023-02-28 (2023 has no 29 February); 2023 is not a leap
        // year and neither is the stub year, so the actual/365 branch applies.
        let yf =
            DayCountConvention::ActualActualAfb.year_fraction(date(2023, 1, 15), date(2024, 2, 29));
        assert_close(yf, 410.0 / 365.0);
    }

    #[test]
    fn extreme_dates_do_not_panic() {
        let (min, max) = (Date::MIN, Date::MAX);
        for convention in DayCountConvention::ALL {
            for (a, b) in [(min, max), (max, min), (max, max), (min, min)] {
                assert!(convention.year_fraction(a, b).is_finite(), "{convention}");
            }
        }
    }

    #[test]
    fn display_uses_market_names() {
        assert_eq!(DayCountConvention::Actual365Fixed.to_string(), "Act/365F");
        assert_eq!(DayCountConvention::ThirtyEPlus360.to_string(), "30E+/360");
    }
}
