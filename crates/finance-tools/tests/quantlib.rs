//! Property tests against QuantLib, called live through `quantlib-ffi`.
//!
//! Every day counter is compared with QuantLib on randomly generated dates
//! (and reference periods), with generation biased towards the dates day
//! count conventions treat specially: month ends, the 28th–31st, February in
//! leap and non-leap years, year ends and century years. For each input the
//! name, the day count and the year fraction must match, and an input QuantLib
//! rejects must be rejected here too.
//!
//! Dates stay within 1901–2150: QuantLib's dates end in 2199 and some
//! conventions step a year or more past the end date.

use finance_tools::jiff::ToSpan;
use finance_tools::jiff::civil::{Date, date};
use finance_tools::{
    Actual365FixedConvention, ActualActualConvention, DayCounter, Thirty360Convention,
};
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use quantlib_ffi as ql;

const CASES: u32 = 20_000;

/// Relative tolerance; differences come only from floating point operation
/// order (and, on some targets, the C++ compiler fusing multiply-adds).
const TOLERANCE: f64 = 1e-12;

const FIRST_YEAR: i16 = 1901;
const LAST_YEAR: i16 = 2150;

// ─── Mapping to QuantLib ────────────────────────────────────────────────────

fn ql_date(d: Date) -> ql::Date {
    ql::Date {
        year: d.year().into(),
        month: d.month().into(),
        day: d.day().into(),
    }
}

fn ql_day_counter(dc: DayCounter) -> ql::DayCounter {
    use ql::DayCounter as Q;
    match dc {
        DayCounter::Actual360 { include_last_day } => Q::Actual360 { include_last_day },
        DayCounter::Actual364 => Q::Actual364,
        DayCounter::Actual366 { include_last_day } => Q::Actual366 { include_last_day },
        DayCounter::Actual36525 { include_last_day } => Q::Actual36525 { include_last_day },
        DayCounter::Actual365Fixed(c) => Q::Actual365Fixed(match c {
            Actual365FixedConvention::Standard => ql::Actual365FixedConvention::Standard,
            Actual365FixedConvention::Canadian => ql::Actual365FixedConvention::Canadian,
            Actual365FixedConvention::NoLeap => ql::Actual365FixedConvention::NoLeap,
        }),
        DayCounter::ActualActual(c) => Q::ActualActual(match c {
            ActualActualConvention::Isma => ql::ActualActualConvention::Isma,
            ActualActualConvention::Bond => ql::ActualActualConvention::Bond,
            ActualActualConvention::Isda => ql::ActualActualConvention::Isda,
            ActualActualConvention::Historical => ql::ActualActualConvention::Historical,
            ActualActualConvention::Actual365 => ql::ActualActualConvention::Actual365,
            ActualActualConvention::Afb => ql::ActualActualConvention::Afb,
            ActualActualConvention::Euro => ql::ActualActualConvention::Euro,
        }),
        DayCounter::Thirty360 {
            convention,
            termination_date,
        } => Q::Thirty360 {
            convention: match convention {
                Thirty360Convention::Usa => ql::Thirty360Convention::Usa,
                Thirty360Convention::BondBasis => ql::Thirty360Convention::BondBasis,
                Thirty360Convention::European => ql::Thirty360Convention::European,
                Thirty360Convention::EurobondBasis => ql::Thirty360Convention::EurobondBasis,
                Thirty360Convention::Italian => ql::Thirty360Convention::Italian,
                Thirty360Convention::German => ql::Thirty360Convention::German,
                Thirty360Convention::Isma => ql::Thirty360Convention::Isma,
                Thirty360Convention::Isda => ql::Thirty360Convention::Isda,
                Thirty360Convention::Nasd => ql::Thirty360Convention::Nasd,
            },
            termination_date: termination_date.map(ql_date),
        },
        DayCounter::Thirty365 => Q::Thirty365,
        DayCounter::OneDayCounter => Q::OneDayCounter,
        DayCounter::SimpleDayCounter => Q::SimpleDayCounter,
    }
}

// ─── Comparison ─────────────────────────────────────────────────────────────

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= TOLERANCE * b.abs().max(1.0)
}

fn check(
    dc: DayCounter,
    d1: Date,
    d2: Date,
    reference: Option<(Date, Date)>,
) -> Result<(), TestCaseError> {
    let q = ql_day_counter(dc);
    let context = format!("{dc:?} {d1} → {d2}, reference {reference:?}");

    prop_assert_eq!(dc.name(), q.name().unwrap(), "{}", context);
    prop_assert_eq!(
        dc.day_count(d1, d2),
        q.day_count(ql_date(d1), ql_date(d2)).unwrap(),
        "day count: {}",
        context
    );

    let ours = match reference {
        None => dc.year_fraction(d1, d2),
        Some((rs, re)) => dc.year_fraction_with_reference_period(d1, d2, rs, re),
    };
    let theirs = q.year_fraction(
        ql_date(d1),
        ql_date(d2),
        reference.map(|(rs, _)| ql_date(rs)),
        reference.map(|(_, re)| ql_date(re)),
    );
    match (ours, theirs) {
        (Ok(ours), Ok(theirs)) => prop_assert!(
            close(ours, theirs),
            "year fraction: {context}: ours {ours}, QuantLib {theirs}"
        ),
        (Err(_), Err(_)) => {}
        (ours, theirs) => {
            return Err(TestCaseError::fail(format!(
                "year fraction: {context}: ours {ours:?}, QuantLib {theirs:?}"
            )));
        }
    }
    Ok(())
}

// ─── Strategies ─────────────────────────────────────────────────────────────

/// Keeps generated dates inside the tested range.
fn in_range(d: Date) -> Date {
    d.clamp(date(FIRST_YEAR, 1, 1), date(LAST_YEAR, 12, 31))
}

fn clamp_to_month(year: i16, month: i8, day: i8) -> Date {
    let first = date(year, month, 1);
    first
        .with()
        .day(day.min(first.days_in_month()))
        .build()
        .unwrap()
}

/// A date in `FIRST_YEAR..=LAST_YEAR`: uniformly random half of the time,
/// otherwise a day that day count conventions treat specially.
fn any_date() -> impl Strategy<Value = Date> {
    let first = date(FIRST_YEAR, 1, 1);
    let days = (date(LAST_YEAR, 12, 31) - first).get_days();
    let uniform = (0..=days).prop_map(move |n| first.checked_add(n.days()).unwrap());
    let special_year = prop_oneof![
        FIRST_YEAR..=LAST_YEAR,
        // Leap years, non-leap years next to them, and century years.
        prop::sample::select(vec![1999_i16, 2000, 2001, 2003, 2004, 2023, 2024, 2100]),
    ];
    let special = (
        special_year,
        1_i8..=12,
        prop::sample::select(vec![1_i8, 15, 27, 28, 29, 30, 31]),
    )
        .prop_map(|(y, m, d)| clamp_to_month(y, m, d));
    prop_oneof![uniform, special]
}

/// A date near `anchor`: the same day, a few days away, or whole months away
/// (so periods often start and end on the same day of the month).
fn near(anchor: Date) -> impl Strategy<Value = Date> {
    prop_oneof![
        Just(anchor),
        (-40_i32..=40).prop_map(move |n| in_range(anchor.saturating_add(n.days()))),
        (-36_i32..=36).prop_map(move |n| in_range(anchor.saturating_add(n.months()))),
        (-3_i32..=3)
            .prop_map(move |n| { in_range(anchor.saturating_add(n.years()).last_of_month()) }),
    ]
}

/// A pair of dates: independent, or close together.
fn date_pair() -> impl Strategy<Value = (Date, Date)> {
    prop_oneof![
        (any_date(), any_date()),
        any_date().prop_flat_map(|d1| (Just(d1), near(d1))),
    ]
}

/// Every day counter variant (30/360 ones without a termination date).
fn all_day_counters() -> Vec<DayCounter> {
    use ActualActualConvention as AA;
    use Thirty360Convention as T;
    let mut all = vec![DayCounter::Actual364, DayCounter::Thirty365];
    all.extend([DayCounter::OneDayCounter, DayCounter::SimpleDayCounter]);
    for include_last_day in [false, true] {
        all.extend([
            DayCounter::Actual360 { include_last_day },
            DayCounter::Actual366 { include_last_day },
            DayCounter::Actual36525 { include_last_day },
        ]);
    }
    all.extend(
        [
            Actual365FixedConvention::Standard,
            Actual365FixedConvention::Canadian,
            Actual365FixedConvention::NoLeap,
        ]
        .map(DayCounter::Actual365Fixed),
    );
    all.extend(
        [
            AA::Isma,
            AA::Bond,
            AA::Isda,
            AA::Historical,
            AA::Actual365,
            AA::Afb,
            AA::Euro,
        ]
        .map(DayCounter::ActualActual),
    );
    all.extend(
        [
            T::Usa,
            T::BondBasis,
            T::European,
            T::EurobondBasis,
            T::Italian,
            T::German,
            T::Isma,
            T::Isda,
            T::Nasd,
        ]
        .map(DayCounter::thirty_360),
    );
    all
}

/// Uniform over every variant, so rare conventions get as many cases as
/// common ones.
fn any_day_counter() -> impl Strategy<Value = DayCounter> {
    prop::sample::select(all_day_counters())
}

/// How the 30E/360 ISDA termination date relates to the end date.
#[derive(Clone, Copy, Debug)]
enum Termination {
    None,
    EndDate,
    Other(Date),
}

fn with_termination(dc: DayCounter, termination: Termination, d2: Date) -> DayCounter {
    match dc {
        DayCounter::Thirty360 { convention, .. } => DayCounter::Thirty360 {
            convention,
            termination_date: match termination {
                Termination::None => None,
                Termination::EndDate => Some(d2),
                Termination::Other(d) => Some(d),
            },
        },
        other => other,
    }
}

fn any_termination() -> impl Strategy<Value = Termination> {
    prop_oneof![
        Just(Termination::None),
        Just(Termination::EndDate),
        any_date().prop_map(Termination::Other),
    ]
}

/// A reference (coupon) period around `d1`: a regular coupon of 1, 3, 6 or
/// 12 months that `d1` falls in or near, or an arbitrary period.
fn reference_period(d1: Date) -> impl Strategy<Value = (Date, Date)> {
    let regular = (
        prop::sample::select(vec![1_i32, 3, 6, 12]),
        -2_i32..=1,
        near(d1),
    )
        .prop_map(|(months, shift, start)| {
            let start = in_range(start.saturating_add((months * shift).months()));
            (start, in_range(start.saturating_add(months.months())))
        });
    let arbitrary = (near(d1), near(d1));
    prop_oneof![regular, arbitrary]
}

// ─── Tests ──────────────────────────────────────────────────────────────────

/// Every pair of the days conventions single out (1st, 15th, 27th–31st, last
/// of month) across leap, non-leap and century years, for every day counter,
/// in both directions. Random cases rarely combine a specific convention with
/// two specific edge days; this does so exhaustively.
#[test]
fn matches_quantlib_on_every_pair_of_edge_days() {
    let mut dates = Vec::new();
    for year in [1999, 2000, 2003, 2004, 2100] {
        for month in 1..=12 {
            for day in [1, 15, 27, 28, 29, 30, 31] {
                dates.push(clamp_to_month(year, month, day));
            }
        }
    }
    dates.sort_unstable();
    dates.dedup();

    for dc in all_day_counters() {
        for &d1 in &dates {
            for &d2 in &dates {
                let mut variants = vec![dc];
                // 30E/360 ISDA also with the end date as termination date.
                if matches!(dc, DayCounter::Thirty360 { .. }) {
                    variants.push(with_termination(dc, Termination::EndDate, d2));
                }
                for dc in variants {
                    if let Err(e) = check(dc, d1, d2, None) {
                        panic!("{e}");
                    }
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn matches_quantlib(
        dc in any_day_counter(),
        (d1, d2) in date_pair(),
        termination in any_termination(),
    ) {
        check(with_termination(dc, termination, d2), d1, d2, None)?;
    }

    #[test]
    fn matches_quantlib_with_reference_period(
        dc in prop_oneof![
            Just(DayCounter::ActualActual(ActualActualConvention::Isma)),
            Just(DayCounter::ActualActual(ActualActualConvention::Bond)),
            Just(DayCounter::Actual365Fixed(Actual365FixedConvention::Canadian)),
            any_day_counter(),
        ],
        ((d1, d2), reference) in date_pair()
            .prop_flat_map(|(d1, d2)| (Just((d1, d2)), reference_period(d1))),
    ) {
        check(dc, d1, d2, Some(reference))?;
    }
}
