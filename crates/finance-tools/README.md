# finance-tools

[QuantLib](https://www.quantlib.org)'s day counters in pure Rust, on
[`jiff`](https://docs.rs/jiff) civil dates. The API mirrors QuantLib's, and
results are property-tested against QuantLib itself.

```rust
use finance_tools::{ActualActualConvention, DayCounter, Thirty360Convention};
use finance_tools::jiff::civil::date;

let dc = DayCounter::ActualActual(ActualActualConvention::Isda);
let yf = dc.year_fraction(date(2003, 11, 1), date(2004, 5, 1))?; // 0.4977…

let bond = DayCounter::thirty_360(Thirty360Convention::BondBasis);
assert_eq!(bond.name(), "30/360 (Bond Basis)");
assert_eq!(bond.day_count(date(2024, 1, 31), date(2024, 3, 31)), 60);
# Ok::<(), finance_tools::Error>(())
```

| QuantLib                              | `finance_tools::DayCounter`                    |
| ------------------------------------- | ---------------------------------------------- |
| `Actual360(includeLastDay)`           | `Actual360 { include_last_day }`               |
| `Actual364()`                         | `Actual364`                                    |
| `Actual366(includeLastDay)`           | `Actual366 { include_last_day }`               |
| `Actual36525(includeLastDay)`         | `Actual36525 { include_last_day }`             |
| `Actual365Fixed(convention)`          | `Actual365Fixed(Actual365FixedConvention)`     |
| `ActualActual(convention)`            | `ActualActual(ActualActualConvention)`         |
| `Thirty360(convention, terminationDate)` | `Thirty360 { convention, termination_date }` |
| `Thirty365()`                         | `Thirty365`                                    |
| `OneDayCounter()`                     | `OneDayCounter`                                |
| `SimpleDayCounter()`                  | `SimpleDayCounter`                             |
| `dc.name()`                           | `dc.name()`                                    |
| `dc.dayCount(d1, d2)`                 | `dc.day_count(d1, d2)`                         |
| `dc.yearFraction(d1, d2)`             | `dc.year_fraction(d1, d2)`                     |
| `dc.yearFraction(d1, d2, refStart, refEnd)` | `dc.year_fraction_with_reference_period(d1, d2, ref_start, ref_end)` |

Every convention of each, aliases included, is supported. Not supported:
`Business252` (needs a holiday calendar) and `ActualActual` with a
`Schedule`. Where QuantLib throws, these return an `Error`.

`jiff` is part of this crate's public API and is re-exported as
`finance_tools::jiff`. Dates are not limited to QuantLib's 1901–2199.

A JavaScript version is published on npm as
[`finance-tools`](https://www.npmjs.com/package/finance-tools).

## Minimum supported Rust version

Rust 1.85 (edition 2024). Raising it is a minor version change.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
