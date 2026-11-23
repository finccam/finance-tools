//! Parity tests against the `RustQuant_time` reference implementation.
//!
//! Every convention is evaluated over a dense grid of date pairs chosen to hit
//! the edge cases day count conventions care about: month ends, 30th/31st,
//! end of February in leap and non-leap years, year boundaries and century
//! leap-year rules.

use RustQuant_time::DayCountConvention as Rq;
use finance_tools::DayCountConvention;
use jiff::civil::{Date, date};

/// Tolerance for floating point differences from operation ordering.
const TOLERANCE: f64 = 1e-12;

fn reference(convention: DayCountConvention) -> Rq {
    match convention {
        DayCountConvention::OneOne => Rq::One_One,
        DayCountConvention::Actual360 => Rq::Actual_360,
        DayCountConvention::Actual364 => Rq::Actual_364,
        DayCountConvention::Actual366 => Rq::Actual_366,
        DayCountConvention::Actual365_25 => Rq::Actual_365_25,
        DayCountConvention::Actual365Actual => Rq::Actual_365_Actual,
        DayCountConvention::Actual365Fixed => Rq::Actual_365_Fixed,
        DayCountConvention::Actual365Leap => Rq::Actual_365_Leap,
        DayCountConvention::ActualActualAfb => Rq::Actual_Actual_AFB,
        DayCountConvention::ActualActualIsda => Rq::Actual_Actual_ISDA,
        DayCountConvention::NoLeap360 => Rq::No_Leap_360,
        DayCountConvention::NoLeap365 => Rq::No_Leap_365,
        DayCountConvention::Thirty360Isda => Rq::Thirty_360_ISDA,
        DayCountConvention::ThirtyE360 => Rq::Thirty_E_360,
        DayCountConvention::ThirtyE360Isda => Rq::Thirty_E_360_ISDA,
        DayCountConvention::ThirtyE365 => Rq::Thirty_E_365,
        DayCountConvention::ThirtyEPlus360 => Rq::Thirty_E_Plus_360,
        DayCountConvention::ThirtyU360 => Rq::Thirty_U_360,
    }
}

fn to_time(date: Date) -> time::Date {
    let month = time::Month::try_from(u8::try_from(date.month()).unwrap()).unwrap();
    time::Date::from_calendar_date(date.year().into(), month, u8::try_from(date.day()).unwrap())
        .unwrap()
}

/// `RustQuant`'s Act/Act AFB unwraps an invalid date when the end date is
/// 29 February and the stub year has no 29 February. We define that case
/// (see the crate docs), so it is excluded from parity.
fn rustquant_panics(convention: DayCountConvention, start: Date, end: Date) -> bool {
    convention == DayCountConvention::ActualActualAfb
        && end.month() == 2
        && end.day() == 29
        && (!start.in_leap_year() || (start.month(), start.day()) > (2, 29))
}

/// Dates around every edge a day count convention distinguishes.
fn interesting_dates() -> Vec<Date> {
    let mut dates = Vec::new();
    for year in [
        1899, 1900, 1999, 2000, 2001, 2003, 2004, 2023, 2024, 2025, 2100,
    ] {
        for month in 1..=12 {
            let first = date(year, month, 1);
            let last = first.last_of_month();
            dates.extend([first, date(year, month, 15), last]);
            if last.day() == 31 {
                dates.push(date(year, month, 30));
            }
            if month == 2 {
                dates.push(date(year, 2, 28));
            }
        }
    }
    dates.sort_unstable();
    dates.dedup();
    dates
}

#[test]
fn matches_rustquant_on_edge_case_grid() {
    let dates = interesting_dates();
    let mut compared = 0_usize;

    for convention in DayCountConvention::ALL {
        let rq = reference(convention);
        for &start in &dates {
            for &end in &dates {
                if rustquant_panics(convention, start, end) {
                    continue;
                }
                let ours = convention.year_fraction(start, end);
                let theirs = rq.day_count_factor(to_time(start), to_time(end));
                assert!(
                    (ours - theirs).abs() <= TOLERANCE,
                    "{convention} {start} -> {end}: ours {ours}, RustQuant {theirs}",
                );
                compared += 1;
            }
        }
    }

    // Guard against the grid silently shrinking.
    assert!(compared > 1_000_000, "only {compared} comparisons ran");
}

#[test]
fn matches_rustquant_on_every_day_of_a_leap_cycle() {
    // Every start date across a full leap cycle against a set of fixed ends,
    // in both directions.
    let ends = [
        date(2024, 2, 28),
        date(2024, 3, 31),
        date(2025, 2, 28),
        date(2027, 8, 31),
        date(2029, 12, 31),
    ];

    for convention in DayCountConvention::ALL {
        let rq = reference(convention);
        let mut day = date(2023, 1, 1);
        while day < date(2028, 1, 1) {
            for end in ends {
                for (a, b) in [(day, end), (end, day)] {
                    if rustquant_panics(convention, a, b) {
                        continue;
                    }
                    let ours = convention.year_fraction(a, b);
                    let theirs = rq.day_count_factor(to_time(a), to_time(b));
                    assert!(
                        (ours - theirs).abs() <= TOLERANCE,
                        "{convention} {a} -> {b}: ours {ours}, RustQuant {theirs}",
                    );
                }
            }
            day = day.tomorrow().unwrap();
        }
    }
}
