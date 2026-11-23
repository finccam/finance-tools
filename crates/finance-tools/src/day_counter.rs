//! Day counters, mirroring QuantLib's `ql/time/daycounters`.
//!
//! A day counter maps a pair of dates to a day count and a *year fraction*
//! (the time, in years, used to accrue interest over the period).
//!
//! Every QuantLib day counter that can be evaluated from dates alone is
//! here, with the same constructors, the same conventions (aliases
//! included), the same names and the same results. The tests in
//! `tests/quantlib.rs` check this against QuantLib 1.44 over randomly
//! generated dates and reference periods.
//!
//! Not included:
//!
//! - `Business252`, which needs a holiday calendar.
//! - `ActualActual` with a `Schedule`. The schedule-free form, where the
//!   coupon period is passed as a reference period, is supported.
//!
//! QuantLib's dates are limited to 1901–2199. These implementations work over
//! the whole [`jiff::civil::Date`] range, following the same formulas.

use core::fmt;

use jiff::Span;
use jiff::civil::Date;

/// Conventions of [`DayCounter::Actual365Fixed`]
/// (`QuantLib::Actual365Fixed::Convention`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Actual365FixedConvention {
    /// Actual days divided by 365.
    #[default]
    Standard,
    /// Canadian bond convention: needs the coupon period as the reference
    /// period, and gives fractions of a coupon for periods of at least a full
    /// coupon.
    Canadian,
    /// "No leap": 29 February is not counted.
    NoLeap,
}

/// Conventions of [`DayCounter::ActualActual`]
/// (`QuantLib::ActualActual::Convention`).
///
/// Several are aliases of one another, exactly as in QuantLib.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ActualActualConvention {
    /// ICMA (formerly ISMA): actual days over the actual days of the
    /// reference (coupon) period, times the period's length in years. Without
    /// a reference period, the accrual period itself is used.
    Isma,
    /// Alias of [`Isma`](Self::Isma).
    Bond,
    /// ISDA: days in leap years over 366, other days over 365.
    Isda,
    /// Alias of [`Isda`](Self::Isda).
    Historical,
    /// Alias of [`Isda`](Self::Isda).
    Actual365,
    /// AFB (Association Française des Banques): whole years counted back from
    /// the end date, then the remaining stub over 366 if it contains
    /// 29 February and 365 otherwise.
    Afb,
    /// Alias of [`Afb`](Self::Afb).
    Euro,
}

/// Conventions of [`DayCounter::Thirty360`] (`QuantLib::Thirty360::Convention`).
///
/// Several are aliases of one another, exactly as in QuantLib.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Thirty360Convention {
    /// 30/360 US (also "30/360 ISDA" in some sources, but not QuantLib's
    /// [`Isda`](Self::Isda)): end-of-February rules for both dates.
    Usa,
    /// Bond basis: start 31 → 30; end 31 → 30 if the start is now 30.
    BondBasis,
    /// 30E/360: any 31 → 30.
    European,
    /// Alias of [`European`](Self::European).
    EurobondBasis,
    /// Italian: as European, and any date in February after the 27th → 30.
    Italian,
    /// Alias of [`Isda`](Self::Isda).
    German,
    /// Alias of [`BondBasis`](Self::BondBasis).
    Isma,
    /// 30E/360 ISDA: any 31 or last day of February → 30, except that the
    /// end date keeps a last-of-February day if it is the termination date.
    Isda,
    /// NASD: as bond basis, but an end 31 with a start before the 30th rolls
    /// to the 1st of the next month.
    Nasd,
}

/// A day counter (`QuantLib::DayCounter`), by its QuantLib constructor.
///
/// Parameters that QuantLib gives defaults have the same defaults in the
/// helper constructors such as [`DayCounter::actual_360`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DayCounter {
    /// Actual/360: actual days (plus one if `include_last_day`) over 360.
    Actual360 {
        /// Count the end date as a full day.
        include_last_day: bool,
    },
    /// Actual/364: actual days over 364.
    Actual364,
    /// Actual/366: actual days (plus one if `include_last_day`) over 366.
    Actual366 {
        /// Count the end date as a full day.
        include_last_day: bool,
    },
    /// Actual/365.25: actual days (plus one if `include_last_day`) over
    /// 365.25.
    Actual36525 {
        /// Count the end date as a full day.
        include_last_day: bool,
    },
    /// Actual/365 (Fixed) and its variants.
    Actual365Fixed(Actual365FixedConvention),
    /// Actual/Actual and its variants.
    ActualActual(ActualActualConvention),
    /// 30/360 and its variants.
    Thirty360 {
        /// The 30/360 variant.
        convention: Thirty360Convention,
        /// Only used by [`Thirty360Convention::Isda`] and its alias
        /// [`German`](Thirty360Convention::German): an end date on the last
        /// day of February is not moved to the 30th if it is this date.
        termination_date: Option<Date>,
    },
    /// 30/365: any 31 → 30, then days over 365.
    Thirty365,
    /// 1/1: the year fraction is always 1 (or −1 for reversed dates).
    OneDayCounter,
    /// Simple: whole months count as twelfths of a year; anything else falls
    /// back to 30/360 bond basis.
    SimpleDayCounter,
}

/// Why a year fraction could not be computed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Error {
    /// The day counter needs a reference period and none was given.
    MissingReferencePeriod,
    /// The reference period cannot be used with these dates.
    InvalidReferencePeriod,
    /// Date arithmetic left the range of [`jiff::civil::Date`].
    DateOutOfRange,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingReferencePeriod => "this day counter needs a reference period",
            Self::InvalidReferencePeriod => "invalid reference period for these dates",
            Self::DateOutOfRange => "date arithmetic left the supported date range",
        })
    }
}

impl core::error::Error for Error {}

impl DayCounter {
    /// `Actual360()`.
    #[must_use]
    pub const fn actual_360() -> Self {
        Self::Actual360 {
            include_last_day: false,
        }
    }

    /// `Actual366()`.
    #[must_use]
    pub const fn actual_366() -> Self {
        Self::Actual366 {
            include_last_day: false,
        }
    }

    /// `Actual36525()`.
    #[must_use]
    pub const fn actual_36525() -> Self {
        Self::Actual36525 {
            include_last_day: false,
        }
    }

    /// `Actual365Fixed()`.
    #[must_use]
    pub const fn actual_365_fixed() -> Self {
        Self::Actual365Fixed(Actual365FixedConvention::Standard)
    }

    /// `Thirty360(convention)`, without a termination date.
    #[must_use]
    pub const fn thirty_360(convention: Thirty360Convention) -> Self {
        Self::Thirty360 {
            convention,
            termination_date: None,
        }
    }

    /// QuantLib's name for this day counter, e.g. `"Actual/365 (Fixed)"` or
    /// `"30E/360 (Eurobond Basis)"`. Aliases share a name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        use ActualActualConvention as AA;
        use Thirty360Convention as T;
        match self {
            Self::Actual360 { include_last_day } => {
                if include_last_day {
                    "Actual/360 (inc)"
                } else {
                    "Actual/360"
                }
            }
            Self::Actual364 => "Actual/364",
            Self::Actual366 { include_last_day } => {
                if include_last_day {
                    "Actual/366 (inc)"
                } else {
                    "Actual/366"
                }
            }
            Self::Actual36525 { include_last_day } => {
                if include_last_day {
                    "Actual/365.25 (inc)"
                } else {
                    "Actual/365.25"
                }
            }
            Self::Actual365Fixed(c) => match c {
                Actual365FixedConvention::Standard => "Actual/365 (Fixed)",
                Actual365FixedConvention::Canadian => "Actual/365 (Fixed) Canadian Bond",
                Actual365FixedConvention::NoLeap => "Actual/365 (No Leap)",
            },
            Self::ActualActual(c) => match c {
                AA::Isma | AA::Bond => "Actual/Actual (ISMA)",
                AA::Isda | AA::Historical | AA::Actual365 => "Actual/Actual (ISDA)",
                AA::Afb | AA::Euro => "Actual/Actual (AFB)",
            },
            Self::Thirty360 { convention, .. } => match convention {
                T::Usa => "30/360 (US)",
                T::BondBasis | T::Isma => "30/360 (Bond Basis)",
                T::European | T::EurobondBasis => "30E/360 (Eurobond Basis)",
                T::Italian => "30/360 (Italian)",
                T::Isda | T::German => "30E/360 (ISDA)",
                T::Nasd => "30/360 (NASD)",
            },
            Self::Thirty365 => "30/365",
            Self::OneDayCounter => "1/1",
            Self::SimpleDayCounter => "Simple",
        }
    }

    /// The number of days between two dates according to this day counter
    /// (`DayCounter::dayCount`). Negative if `d2` is before `d1`, except where
    /// the convention's formula says otherwise.
    #[must_use]
    pub fn day_count(self, d1: Date, d2: Date) -> i64 {
        match self {
            Self::Actual360 { include_last_day }
            | Self::Actual366 { include_last_day }
            | Self::Actual36525 { include_last_day } => {
                days_between(d1, d2) + i64::from(include_last_day)
            }
            Self::Actual365Fixed(Actual365FixedConvention::NoLeap) => no_leap_day_count(d1, d2),
            Self::Actual364 | Self::Actual365Fixed(_) | Self::ActualActual(_) => {
                days_between(d1, d2)
            }
            Self::Thirty360 {
                convention,
                termination_date,
            } => thirty_360_day_count(convention, termination_date, d1, d2),
            Self::Thirty365 => thirty_365_day_count(d1, d2),
            Self::OneDayCounter => {
                if d2 >= d1 {
                    1
                } else {
                    -1
                }
            }
            Self::SimpleDayCounter => {
                thirty_360_day_count(Thirty360Convention::BondBasis, None, d1, d2)
            }
        }
    }

    /// The year fraction between two dates (`DayCounter::yearFraction(d1,
    /// d2)`).
    ///
    /// # Errors
    ///
    /// [`Error::MissingReferencePeriod`] for
    /// [`Actual365FixedConvention::Canadian`], which always needs one (unless
    /// the dates are equal); see
    /// [`year_fraction_with_reference_period`](Self::year_fraction_with_reference_period).
    /// Otherwise, as that method.
    pub fn year_fraction(self, d1: Date, d2: Date) -> Result<f64, Error> {
        self.year_fraction_impl(d1, d2, None, None)
    }

    /// The year fraction between two dates, given the reference (coupon)
    /// period they accrue in (`DayCounter::yearFraction(d1, d2, refPeriodStart,
    /// refPeriodEnd)`).
    ///
    /// Only [`ActualActualConvention::Isma`] (and its alias
    /// [`Bond`](ActualActualConvention::Bond)) and
    /// [`Actual365FixedConvention::Canadian`] use the reference period; other
    /// day counters ignore it.
    ///
    /// # Errors
    ///
    /// - [`Error::InvalidReferencePeriod`] if the reference period does not
    ///   fit the dates, under the same rules as QuantLib.
    /// - [`Error::DateOutOfRange`] if stepping through coupon periods leaves
    ///   the [`jiff::civil::Date`] range.
    pub fn year_fraction_with_reference_period(
        self,
        d1: Date,
        d2: Date,
        ref_period_start: Date,
        ref_period_end: Date,
    ) -> Result<f64, Error> {
        self.year_fraction_impl(d1, d2, Some(ref_period_start), Some(ref_period_end))
    }

    // Day counts are far below 2^52, so the `i64` → `f64` conversions below
    // are exact.
    #[allow(clippy::cast_precision_loss)]
    fn year_fraction_impl(
        self,
        d1: Date,
        d2: Date,
        ref_start: Option<Date>,
        ref_end: Option<Date>,
    ) -> Result<f64, Error> {
        use ActualActualConvention as AA;
        let actual = |denominator: f64| self.day_count(d1, d2) as f64 / denominator;
        Ok(match self {
            Self::Actual360 { .. } | Self::Thirty360 { .. } => actual(360.0),
            Self::Actual364 => actual(364.0),
            Self::Actual366 { .. } => actual(366.0),
            Self::Actual36525 { .. } => actual(365.25),
            Self::Actual365Fixed(Actual365FixedConvention::Canadian) => {
                canadian_year_fraction(d1, d2, ref_start, ref_end)?
            }
            Self::Actual365Fixed(_) | Self::Thirty365 => actual(365.0),
            Self::ActualActual(AA::Isma | AA::Bond) => {
                isma_year_fraction(d1, d2, ref_start, ref_end)?
            }
            Self::ActualActual(AA::Isda | AA::Historical | AA::Actual365) => {
                isda_year_fraction(d1, d2)
            }
            Self::ActualActual(AA::Afb | AA::Euro) => afb_year_fraction(d1, d2)?,
            Self::OneDayCounter => actual(1.0),
            Self::SimpleDayCounter => simple_year_fraction(d1, d2),
        })
    }
}

impl fmt::Display for DayCounter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// `d2 - d1` in days.
fn days_between(d1: Date, d2: Date) -> i64 {
    i64::from((d2 - d1).get_days())
}

fn add_months(date: Date, months: i64) -> Result<Date, Error> {
    let span = Span::new()
        .try_months(months)
        .map_err(|_| Error::DateOutOfRange)?;
    date.checked_add(span).map_err(|_| Error::DateOutOfRange)
}

fn add_years(date: Date, years: i64) -> Result<Date, Error> {
    let span = Span::new()
        .try_years(years)
        .map_err(|_| Error::DateOutOfRange)?;
    date.checked_add(span).map_err(|_| Error::DateOutOfRange)
}

fn ymd(date: Date) -> (i64, i64, i64) {
    (
        i64::from(date.year()),
        i64::from(date.month()),
        i64::from(date.day()),
    )
}

fn is_last_of_february(date: Date) -> bool {
    date.month() == 2 && date.day() == date.days_in_month()
}

fn thirty_360_day_count(
    convention: Thirty360Convention,
    termination_date: Option<Date>,
    d1: Date,
    d2: Date,
) -> i64 {
    use Thirty360Convention as T;
    let (yy1, mm1, mut dd1) = ymd(d1);
    let (yy2, mut mm2, mut dd2) = ymd(d2);
    match convention {
        T::Usa => {
            if is_last_of_february(d1) {
                if is_last_of_february(d2) {
                    dd2 = 30;
                }
                dd1 = 30;
            }
            if dd2 == 31 && dd1 >= 30 {
                dd2 = 30;
            }
            if dd1 == 31 {
                dd1 = 30;
            }
        }
        T::BondBasis | T::Isma => {
            if dd1 == 31 {
                dd1 = 30;
            }
            if dd2 == 31 && dd1 == 30 {
                dd2 = 30;
            }
        }
        T::European | T::EurobondBasis => {
            if dd1 == 31 {
                dd1 = 30;
            }
            if dd2 == 31 {
                dd2 = 30;
            }
        }
        T::Italian => {
            if dd1 == 31 {
                dd1 = 30;
            }
            if dd2 == 31 {
                dd2 = 30;
            }
            if mm1 == 2 && dd1 > 27 {
                dd1 = 30;
            }
            if mm2 == 2 && dd2 > 27 {
                dd2 = 30;
            }
        }
        T::Isda | T::German => {
            if dd1 == 31 {
                dd1 = 30;
            }
            if dd2 == 31 {
                dd2 = 30;
            }
            if is_last_of_february(d1) {
                dd1 = 30;
            }
            if Some(d2) != termination_date && is_last_of_february(d2) {
                dd2 = 30;
            }
        }
        T::Nasd => {
            if dd1 == 31 {
                dd1 = 30;
            }
            if dd2 == 31 && dd1 >= 30 {
                dd2 = 30;
            }
            if dd2 == 31 && dd1 < 30 {
                dd2 = 1;
                mm2 += 1;
            }
        }
    }
    360 * (yy2 - yy1) + 30 * (mm2 - mm1) + (dd2 - dd1)
}

fn thirty_365_day_count(d1: Date, d2: Date) -> i64 {
    let (yy1, mm1, mut dd1) = ymd(d1);
    let (yy2, mm2, mut dd2) = ymd(d2);
    if dd1 == 31 {
        dd1 = 30;
    }
    if dd2 == 31 {
        dd2 = 30;
    }
    360 * (yy2 - yy1) + 30 * (mm2 - mm1) + (dd2 - dd1)
}

fn no_leap_day_count(d1: Date, d2: Date) -> i64 {
    const MONTH_OFFSET: [i64; 12] = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let serial = |d: Date| {
        let (y, m, day) = ymd(d);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let offset = MONTH_OFFSET[(m - 1) as usize];
        let s = day + offset + y * 365;
        if m == 2 && day == 29 { s - 1 } else { s }
    };
    serial(d2) - serial(d1)
}

/// `std::lround` of a finite value well inside the `i64` range.
#[allow(clippy::cast_possible_truncation)]
fn lround(x: f64) -> i64 {
    x.round() as i64
}

#[allow(clippy::cast_precision_loss)]
fn canadian_year_fraction(
    d1: Date,
    d2: Date,
    ref_start: Option<Date>,
    ref_end: Option<Date>,
) -> Result<f64, Error> {
    if d1 == d2 {
        return Ok(0.0);
    }
    let (Some(ref_start), Some(ref_end)) = (ref_start, ref_end) else {
        return Err(Error::MissingReferencePeriod);
    };
    let dcs = days_between(d1, d2) as f64;
    let dcc = days_between(ref_start, ref_end) as f64;
    let months = lround(12.0 * dcc / 365.0);
    if months == 0 {
        return Err(Error::InvalidReferencePeriod);
    }
    let frequency = 12 / months;
    if frequency == 0 {
        return Err(Error::InvalidReferencePeriod);
    }
    if dcs < (365 / frequency) as f64 {
        return Ok(dcs / 365.0);
    }
    Ok(1.0 / frequency as f64 - (dcc - dcs) / 365.0)
}

/// QuantLib's `ActualActual::Old_ISMA_Impl::yearFraction`.
///
/// QuantLib handles a long first coupon by recursing once per coupon period
/// back from the reference period. That recursion is unrolled into a loop
/// here so that distant dates cannot overflow the stack; the pieces are added
/// in the same order, so results are identical.
#[allow(clippy::cast_precision_loss)]
fn isma_year_fraction(
    d1: Date,
    d2: Date,
    ref_start: Option<Date>,
    ref_end: Option<Date>,
) -> Result<f64, Error> {
    if d1 == d2 {
        return Ok(0.0);
    }
    if d1 > d2 {
        return Ok(-isma_year_fraction(d2, d1, ref_start, ref_end)?);
    }

    // Without a reference period, take it equal to the accrual period.
    let mut d2 = d2;
    let mut ref_start = ref_start.unwrap_or(d1);
    let mut ref_end = ref_end.unwrap_or(d2);
    // Later pieces of a long first coupon, latest first.
    let mut later = Vec::new();

    let first = loop {
        if !(ref_end > ref_start && ref_end > d1) {
            return Err(Error::InvalidReferencePeriod);
        }

        // Roughly the length of the reference period in months.
        let mut months = lround(12.0 * days_between(ref_start, ref_end) as f64 / 365.0);
        if months == 0 {
            // Short periods: take the reference period as one year from d1.
            ref_start = d1;
            ref_end = add_years(d1, 1)?;
            months = 12;
        }
        let period = months as f64 / 12.0;

        if d2 > ref_end {
            // ref_start <= d1 < ref_end < d2
            if ref_start > d1 {
                return Err(Error::InvalidReferencePeriod);
            }
            break regular_periods_year_fraction(d1, d2, ref_start, ref_end, months, period)?;
        }
        if d1 >= ref_start {
            // ref_start <= d1 <= d2 <= ref_end
            break period * days_between(d1, d2) as f64 / days_between(ref_start, ref_end) as f64;
        }

        // Long first coupon: d1 < ref_start < ref_end, d2 <= ref_end. Continue
        // with the previous (notional) coupon period.
        let previous_ref = add_months(ref_start, -months)?;
        if d2 > ref_start {
            later.push(isma_year_fraction(
                ref_start,
                d2,
                Some(ref_start),
                Some(ref_end),
            )?);
            d2 = ref_start;
        }
        ref_end = ref_start;
        ref_start = previous_ref;
    };

    Ok(later
        .into_iter()
        .rev()
        .fold(first, |sum, piece| sum + piece))
}

/// The `d2 > refPeriodEnd` branch of QuantLib's Old ISMA implementation: the
/// rest of the reference period, whole regular periods, then the remainder.
fn regular_periods_year_fraction(
    d1: Date,
    d2: Date,
    ref_start: Date,
    ref_end: Date,
    months: i64,
    period: f64,
) -> Result<f64, Error> {
    let mut sum = isma_year_fraction(d1, ref_end, Some(ref_start), Some(ref_end))?;
    let mut i = 0;
    let (new_ref_start, new_ref_end) = loop {
        let start = add_months(ref_end, months * i)?;
        let end = add_months(ref_end, months * (i + 1))?;
        if d2 < end {
            break (start, end);
        }
        sum += period;
        i += 1;
    };
    sum += isma_year_fraction(new_ref_start, d2, Some(new_ref_start), Some(new_ref_end))?;
    Ok(sum)
}

/// QuantLib's `ActualActual::ISDA_Impl::yearFraction`.
#[allow(clippy::cast_precision_loss)]
fn isda_year_fraction(d1: Date, d2: Date) -> f64 {
    if d1 == d2 {
        return 0.0;
    }
    if d1 > d2 {
        return -isda_year_fraction(d2, d1);
    }
    let days_in_year = |d: Date| if d.in_leap_year() { 366.0 } else { 365.0 };
    let (y1, y2) = (i64::from(d1.year()), i64::from(d2.year()));
    // Days from d1 to 1 January of the next year, and from 1 January of
    // d2's year to d2.
    let to_year_end = i64::from(d1.days_in_year()) - i64::from(d1.day_of_year()) + 1;
    let from_year_start = i64::from(d2.day_of_year()) - 1;
    let mut sum = (y2 - y1 - 1) as f64;
    sum += to_year_end as f64 / days_in_year(d1);
    sum += from_year_start as f64 / days_in_year(d2);
    sum
}

/// QuantLib's `ActualActual::AFB_Impl::yearFraction`.
#[allow(clippy::cast_precision_loss)]
fn afb_year_fraction(d1: Date, d2: Date) -> Result<f64, Error> {
    if d1 == d2 {
        return Ok(0.0);
    }
    if d1 > d2 {
        return Ok(-afb_year_fraction(d2, d1)?);
    }
    let mut new_d2 = d2;
    let mut temp = d2;
    let mut sum = 0.0;
    while temp > d1 {
        temp = add_years(new_d2, -1)?;
        if temp.day() == 28 && temp.month() == 2 && temp.in_leap_year() {
            temp = temp.tomorrow().map_err(|_| Error::DateOutOfRange)?;
        }
        if temp >= d1 {
            sum += 1.0;
            new_d2 = temp;
        }
    }

    let contains_leap_day = |year: i16| {
        let leap_day = jiff::civil::date(year, 2, 29);
        new_d2 > leap_day && d1 <= leap_day
    };
    let mut den = 365.0;
    if new_d2.in_leap_year() {
        if contains_leap_day(new_d2.year()) {
            den += 1.0;
        }
    } else if d1.in_leap_year() && contains_leap_day(d1.year()) {
        den += 1.0;
    }
    Ok(sum + days_between(d1, new_d2) as f64 / den)
}

/// QuantLib's `SimpleDayCounter::Impl::yearFraction`.
#[allow(clippy::cast_precision_loss)]
fn simple_year_fraction(d1: Date, d2: Date) -> f64 {
    let (dm1, dm2) = (d1.day(), d2.day());
    let is_end_of_month = |d: Date| d.day() == d.days_in_month();
    if dm1 == dm2 || (dm1 > dm2 && is_end_of_month(d2)) || (dm1 < dm2 && is_end_of_month(d1)) {
        let years = i64::from(d2.year()) - i64::from(d1.year());
        let months = i64::from(d2.month()) - i64::from(d1.month());
        years as f64 + months as f64 / 12.0
    } else {
        thirty_360_day_count(Thirty360Convention::BondBasis, None, d1, d2) as f64 / 360.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 1e-11,
            "expected {expected}, got {actual}"
        );
    }

    /// The worked examples of the ISDA paper "EMU and market conventions:
    /// Recent developments" (1998), as used in QuantLib's own test suite:
    /// (start, end, reference start, reference end, ISDA, ISMA, AFB).
    #[test]
    fn isda_paper_examples() {
        #[allow(clippy::type_complexity)]
        let cases: [(
            (i16, i8, i8),
            (i16, i8, i8),
            (i16, i8, i8),
            (i16, i8, i8),
            [f64; 3],
        ); 6] = [
            (
                (2003, 11, 1),
                (2004, 5, 1),
                (2003, 11, 1),
                (2004, 5, 1),
                [0.497_724_380_567, 0.5, 0.497_267_759_563],
            ),
            (
                (1999, 2, 1),
                (1999, 7, 1),
                (1998, 7, 1),
                (1999, 7, 1),
                [0.410_958_904_110, 0.410_958_904_110, 0.410_958_904_110],
            ),
            (
                (2002, 8, 15),
                (2003, 7, 15),
                (2003, 1, 15),
                (2003, 7, 15),
                [0.915_068_493_151, 0.915_760_869_565, 0.915_068_493_151],
            ),
            (
                (2003, 7, 15),
                (2004, 1, 15),
                (2003, 7, 15),
                (2004, 1, 15),
                [0.504_004_790_778, 0.5, 0.504_109_589_041],
            ),
            (
                (1999, 7, 30),
                (2000, 1, 30),
                (1999, 7, 30),
                (2000, 1, 30),
                [0.503_892_506_924, 0.5, 0.504_109_589_041],
            ),
            (
                (2000, 1, 30),
                (2000, 6, 30),
                (2000, 1, 30),
                (2000, 7, 30),
                [0.415_300_546_448, 0.417_582_417_582, 0.415_300_546_448],
            ),
        ];
        let d = |(y, m, day): (i16, i8, i8)| date(y, m, day);
        for (start, end, rs, re, [isda, isma, afb]) in cases {
            let (start, end, rs, re) = (d(start), d(end), d(rs), d(re));
            let yf = |c| {
                DayCounter::ActualActual(c)
                    .year_fraction_with_reference_period(start, end, rs, re)
                    .unwrap()
            };
            assert_close(yf(ActualActualConvention::Isda), isda);
            assert_close(yf(ActualActualConvention::Isma), isma);
            assert_close(yf(ActualActualConvention::Afb), afb);
        }
    }

    #[test]
    fn thirty_360_end_of_month_rules() {
        let yf = |c, d1, d2| DayCounter::thirty_360(c).day_count(d1, d2);
        let (feb_end, mar_31) = (date(2024, 2, 29), date(2024, 3, 31));
        assert_eq!(yf(Thirty360Convention::Usa, feb_end, mar_31), 30);
        assert_eq!(yf(Thirty360Convention::BondBasis, feb_end, mar_31), 32);
        assert_eq!(yf(Thirty360Convention::European, feb_end, mar_31), 31);
        assert_eq!(yf(Thirty360Convention::Isda, feb_end, mar_31), 30);
        assert_eq!(yf(Thirty360Convention::Nasd, date(2024, 3, 15), mar_31), 16);
    }

    #[test]
    fn isda_30e_360_keeps_february_termination_date() {
        let (d1, maturity) = (date(2023, 8, 31), date(2024, 2, 29));
        let with = DayCounter::Thirty360 {
            convention: Thirty360Convention::Isda,
            termination_date: Some(maturity),
        };
        let without = DayCounter::thirty_360(Thirty360Convention::Isda);
        assert_eq!(with.day_count(d1, maturity), 179);
        assert_eq!(without.day_count(d1, maturity), 180);
    }

    #[test]
    fn canadian_needs_a_reference_period() {
        let dc = DayCounter::Actual365Fixed(Actual365FixedConvention::Canadian);
        let (d1, d2) = (date(2024, 1, 1), date(2024, 4, 1));
        assert_eq!(dc.year_fraction(d1, d2), Err(Error::MissingReferencePeriod));
        assert_eq!(dc.year_fraction(d1, d1), Ok(0.0));
        let semiannual = dc
            .year_fraction_with_reference_period(d1, d2, d1, date(2024, 7, 1))
            .unwrap();
        assert_close(semiannual, 91.0 / 365.0);
    }

    #[test]
    fn extreme_dates_do_not_panic() {
        let all = [
            DayCounter::actual_360(),
            DayCounter::Actual364,
            DayCounter::actual_366(),
            DayCounter::actual_36525(),
            DayCounter::actual_365_fixed(),
            DayCounter::Actual365Fixed(Actual365FixedConvention::NoLeap),
            DayCounter::ActualActual(ActualActualConvention::Isma),
            DayCounter::ActualActual(ActualActualConvention::Isda),
            DayCounter::ActualActual(ActualActualConvention::Afb),
            DayCounter::thirty_360(Thirty360Convention::Usa),
            DayCounter::thirty_360(Thirty360Convention::Nasd),
            DayCounter::Thirty365,
            DayCounter::OneDayCounter,
            DayCounter::SimpleDayCounter,
        ];
        let (min, max) = (Date::MIN, Date::MAX);
        for dc in all {
            for (a, b) in [(min, max), (max, min), (max, max), (min, min)] {
                let _ = dc.day_count(a, b);
                if let Ok(yf) = dc.year_fraction(a, b) {
                    assert!(yf.is_finite(), "{dc}");
                }
            }
        }
    }
}
