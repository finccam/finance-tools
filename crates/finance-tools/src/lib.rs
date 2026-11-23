//! Financial calculation primitives.
//!
//! Currently this crate provides [`day_count`], an implementation of the day
//! count conventions supported by
//! [`RustQuant`](https://github.com/avhz/RustQuant), built on
//! [`jiff`] civil dates.
//!
//! ```
//! use finance_tools::DayCountConvention;
//! use jiff::civil::date;
//!
//! let yf = DayCountConvention::Actual360.year_fraction(date(2024, 1, 1), date(2024, 7, 1));
//! assert!((yf - 182.0 / 360.0).abs() < f64::EPSILON);
//! ```
//!
//! [`jiff`] is part of the public API (every date is a
//! [`jiff::civil::Date`]) and is re-exported so callers can name the exact
//! version this crate was built against.

pub mod day_count;

pub use day_count::DayCountConvention;
pub use jiff;
