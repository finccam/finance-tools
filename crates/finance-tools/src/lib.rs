//! Financial calculation primitives.
//!
//! Currently this crate provides [`day_counter`]: QuantLib's day counters,
//! reimplemented in Rust on [`jiff`] civil dates. The API mirrors QuantLib's
//! (`DayCounter`, `name`, `dayCount`, `yearFraction`), and results are
//! property-tested against QuantLib itself.
//!
//! ```
//! use finance_tools::{DayCounter, Thirty360Convention};
//! use finance_tools::jiff::civil::date;
//!
//! let dc = DayCounter::Actual360 { include_last_day: false };
//! let yf = dc.year_fraction(date(2024, 1, 1), date(2024, 7, 1))?;
//! assert_eq!(yf, 182.0 / 360.0);
//!
//! let bond = DayCounter::thirty_360(Thirty360Convention::BondBasis);
//! assert_eq!(bond.name(), "30/360 (Bond Basis)");
//! assert_eq!(bond.day_count(date(2024, 1, 31), date(2024, 3, 31)), 60);
//! # Ok::<(), finance_tools::Error>(())
//! ```
//!
//! [`jiff`] is part of the public API (every date is a
//! [`jiff::civil::Date`]) and is re-exported so callers can name the exact
//! version this crate was built against.

pub mod day_counter;

pub use day_counter::{
    Actual365FixedConvention, ActualActualConvention, DayCounter, Error, Thirty360Convention,
};
pub use jiff;
