# finance-tools

Day count conventions on [`jiff`](https://docs.rs/jiff) civil dates, with
results identical to [`RustQuant_time`](https://docs.rs/RustQuant_time/0.4.0).

```rust
use finance_tools::DayCountConvention;
use finance_tools::jiff::civil::date;

let yf = DayCountConvention::Actual365Fixed.year_fraction(date(2024, 1, 1), date(2024, 7, 1));
```

All conventions implemented by `RustQuant_time` 0.4 are supported, and parity
with it is tested over a dense grid of edge-case dates. Act/Act ICMA is not
supported, because `RustQuant` does not implement it and it needs coupon
schedule information that a two-date API cannot carry.

`jiff` is part of this crate's public API and is re-exported as
`finance_tools::jiff`.

A JavaScript version of this crate is published on npm as
[`finance-tools`](https://www.npmjs.com/package/finance-tools).

## Minimum supported Rust version

Rust 1.85 (edition 2024). Raising it is a minor version change.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
