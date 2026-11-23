//! Minimal FFI to [QuantLib](https://www.quantlib.org)'s day counters.
//!
//! This crate exists only as the reference implementation for
//! `finance-tools`' tests and is never published. It deliberately does not
//! depend on `finance-tools`, so its types are plain mirrors of QuantLib's
//! constructors.
//!
//! QuantLib only represents dates from 1901-01-01 to 2199-12-31; anything
//! outside that range, like any other QuantLib exception, comes back as an
//! [`Error`].

use std::ffi::{CStr, c_char, c_int};
use std::fmt;

/// A calendar date. Not validated here; QuantLib rejects invalid dates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Date {
    /// Year, 1901 to 2199.
    pub year: i32,
    /// Month, 1 to 12.
    pub month: i32,
    /// Day of the month.
    pub day: i32,
}

/// `QuantLib::Actual365Fixed::Convention`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Actual365FixedConvention {
    Standard = 0,
    Canadian = 1,
    NoLeap = 2,
}

/// `QuantLib::ActualActual::Convention`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum ActualActualConvention {
    Isma = 0,
    Bond = 1,
    Isda = 2,
    Historical = 3,
    Actual365 = 4,
    Afb = 5,
    Euro = 6,
}

/// `QuantLib::Thirty360::Convention`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum Thirty360Convention {
    Usa = 0,
    BondBasis = 1,
    European = 2,
    EurobondBasis = 3,
    Italian = 4,
    German = 5,
    Isma = 6,
    Isda = 7,
    Nasd = 8,
}

/// A QuantLib day counter, by its C++ constructor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(missing_docs)]
pub enum DayCounter {
    Actual360 {
        include_last_day: bool,
    },
    Actual364,
    Actual366 {
        include_last_day: bool,
    },
    Actual36525 {
        include_last_day: bool,
    },
    Actual365Fixed(Actual365FixedConvention),
    ActualActual(ActualActualConvention),
    Thirty360 {
        convention: Thirty360Convention,
        termination_date: Option<Date>,
    },
    Thirty365,
    OneDayCounter,
    SimpleDayCounter,
}

/// A QuantLib exception message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QuantLib: {}", self.0)
    }
}

impl std::error::Error for Error {}

mod ffi {
    use std::ffi::{c_char, c_int};

    /// Mirrors `ql_spec` in `shim.cpp`.
    #[repr(C)]
    pub struct Spec {
        pub kind: i32,
        pub convention: i32,
        pub include_last_day: i32,
        pub has_termination_date: i32,
        pub termination_year: i32,
        pub termination_month: i32,
        pub termination_day: i32,
    }

    /// Mirrors `ql_date` in `shim.cpp`.
    #[repr(C)]
    pub struct Date {
        pub year: i32,
        pub month: i32,
        pub day: i32,
    }

    unsafe extern "C" {
        pub fn ql_name(spec: *const Spec, out: *mut c_char, out_len: usize) -> c_int;
        pub fn ql_day_count(
            spec: *const Spec,
            d1: *const Date,
            d2: *const Date,
            out: *mut i64,
            err: *mut c_char,
            err_len: usize,
        ) -> c_int;
        pub fn ql_year_fraction(
            spec: *const Spec,
            d1: *const Date,
            d2: *const Date,
            ref_start: *const Date,
            ref_end: *const Date,
            out: *mut f64,
            err: *mut c_char,
            err_len: usize,
        ) -> c_int;
    }
}

const MESSAGE_LEN: usize = 1024;

impl From<Date> for ffi::Date {
    fn from(d: Date) -> Self {
        Self {
            year: d.year,
            month: d.month,
            day: d.day,
        }
    }
}

impl DayCounter {
    fn spec(self) -> ffi::Spec {
        let (kind, convention, include_last_day, termination) = match self {
            Self::Actual360 { include_last_day } => (0, 0, include_last_day, None),
            Self::Actual364 => (1, 0, false, None),
            Self::Actual366 { include_last_day } => (2, 0, include_last_day, None),
            Self::Actual36525 { include_last_day } => (3, 0, include_last_day, None),
            Self::Actual365Fixed(c) => (4, c as i32, false, None),
            Self::ActualActual(c) => (5, c as i32, false, None),
            Self::Thirty360 {
                convention,
                termination_date,
            } => (6, convention as i32, false, termination_date),
            Self::Thirty365 => (7, 0, false, None),
            Self::OneDayCounter => (8, 0, false, None),
            Self::SimpleDayCounter => (9, 0, false, None),
        };
        let t = termination.unwrap_or(Date {
            year: 0,
            month: 0,
            day: 0,
        });
        ffi::Spec {
            kind,
            convention,
            include_last_day: i32::from(include_last_day),
            has_termination_date: i32::from(termination.is_some()),
            termination_year: t.year,
            termination_month: t.month,
            termination_day: t.day,
        }
    }

    /// `DayCounter::name()`.
    ///
    /// # Errors
    ///
    /// If QuantLib throws, e.g. for an invalid termination date.
    pub fn name(self) -> Result<String, Error> {
        let spec = self.spec();
        let mut buf = [0 as c_char; MESSAGE_LEN];
        // SAFETY: `spec` is a valid `ql_spec`; `buf` is writable for
        // `buf.len()` bytes and the shim always NUL-terminates within it.
        let status = unsafe { ffi::ql_name(&raw const spec, buf.as_mut_ptr(), buf.len()) };
        result(status, &buf, ()).map(|()| message(&buf))
    }

    /// `DayCounter::dayCount(d1, d2)`.
    ///
    /// # Errors
    ///
    /// If QuantLib throws, e.g. for a date outside its range.
    pub fn day_count(self, d1: Date, d2: Date) -> Result<i64, Error> {
        let spec = self.spec();
        let (d1, d2) = (ffi::Date::from(d1), ffi::Date::from(d2));
        let mut out = 0_i64;
        let mut err = [0 as c_char; MESSAGE_LEN];
        // SAFETY: all pointers are to live, properly initialised values of the
        // `#[repr(C)]` types the shim expects; `err` is writable for
        // `err.len()` bytes.
        let status = unsafe {
            ffi::ql_day_count(
                &raw const spec,
                &raw const d1,
                &raw const d2,
                &raw mut out,
                err.as_mut_ptr(),
                err.len(),
            )
        };
        result(status, &err, out)
    }

    /// `DayCounter::yearFraction(d1, d2, refPeriodStart, refPeriodEnd)`;
    /// `None` passes QuantLib's null `Date()`.
    ///
    /// # Errors
    ///
    /// If QuantLib throws, e.g. for a missing or invalid reference period.
    pub fn year_fraction(
        self,
        d1: Date,
        d2: Date,
        ref_start: Option<Date>,
        ref_end: Option<Date>,
    ) -> Result<f64, Error> {
        let spec = self.spec();
        let (d1, d2) = (ffi::Date::from(d1), ffi::Date::from(d2));
        let ref_start = ref_start.map(ffi::Date::from);
        let ref_end = ref_end.map(ffi::Date::from);
        let mut out = 0.0_f64;
        let mut err = [0 as c_char; MESSAGE_LEN];
        // SAFETY: as in `day_count`; the reference dates are either null or
        // point to live `ffi::Date`s owned by this frame.
        let status = unsafe {
            ffi::ql_year_fraction(
                &raw const spec,
                &raw const d1,
                &raw const d2,
                ref_start
                    .as_ref()
                    .map_or(std::ptr::null(), std::ptr::from_ref),
                ref_end
                    .as_ref()
                    .map_or(std::ptr::null(), std::ptr::from_ref),
                &raw mut out,
                err.as_mut_ptr(),
                err.len(),
            )
        };
        result(status, &err, out)
    }
}

fn message(buf: &[c_char]) -> String {
    // SAFETY: the shim NUL-terminates every message within the buffer, and
    // the buffer is zero-initialised, so a NUL is always present.
    unsafe { CStr::from_ptr(buf.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

fn result<T>(status: c_int, err: &[c_char], value: T) -> Result<T, Error> {
    if status == 0 {
        Ok(value)
    } else {
        Err(Error(message(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_into_quantlib() {
        let dc = DayCounter::ActualActual(ActualActualConvention::Isda);
        assert_eq!(dc.name().unwrap(), "Actual/Actual (ISDA)");
        let (d1, d2) = (
            Date {
                year: 2003,
                month: 11,
                day: 1,
            },
            Date {
                year: 2004,
                month: 5,
                day: 1,
            },
        );
        let yf = dc.year_fraction(d1, d2, None, None).unwrap();
        assert!((yf - 0.497_724_380_567).abs() < 1e-12);
        assert_eq!(dc.day_count(d1, d2).unwrap(), 182);
    }

    #[test]
    fn exceptions_become_errors() {
        let dc = DayCounter::Actual365Fixed(Actual365FixedConvention::Canadian);
        let d = |year| Date {
            year,
            month: 1,
            day: 1,
        };
        assert!(dc.year_fraction(d(2020), d(2021), None, None).is_err());
        assert!(DayCounter::Actual364.day_count(d(1800), d(2000)).is_err());
    }
}
