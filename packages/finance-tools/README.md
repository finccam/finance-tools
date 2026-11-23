# finance-tools

[QuantLib](https://www.quantlib.org)'s day counters for JavaScript, powered by
the [`finance-tools`](https://crates.io/crates/finance-tools) Rust crate
compiled to WebAssembly. The classes mirror QuantLib's Python bindings, and
results are property-tested against QuantLib itself.

```js
import { ActualActual, Thirty360 } from "finance-tools";

const dc = new ActualActual(ActualActual.ISDA);
dc.name(); // "Actual/Actual (ISDA)"
dc.yearFraction("2003-11-01", "2004-05-01"); // 0.4977…

const bond = new Thirty360(Thirty360.BondBasis);
bond.dayCount("2024-01-31", "2024-03-31"); // 60

// ISMA with a reference (coupon) period
new ActualActual(ActualActual.ISMA).yearFraction(
  "2003-11-01", "2004-05-01", "2003-11-01", "2004-05-01",
); // 0.5
```

Available: `Actual360`, `Actual364`, `Actual366`, `Actual36525`,
`Actual365Fixed`, `ActualActual`, `Thirty360`, `Thirty365`, `OneDayCounter`
and `SimpleDayCounter`, with every QuantLib convention of each. `Business252`
and `ActualActual` with a schedule are not available.

Dates are ISO 8601 calendar dates (`YYYY-MM-DD`). JavaScript `Date` objects
are not accepted, because they are instants and turning one into a calendar
date depends on a time zone. Methods throw where QuantLib would, and on
invalid dates.

The package is ESM-only and uses the WebAssembly ESM integration. It works
with bundlers (Vite, webpack, Rollup with a wasm plugin) and directly in
Node.js ≥ 24.5.

## Development

The JavaScript API lives in `src/index.ts`. The `wasm-bindgen` glue it wraps
is generated from [`bindings/wasm`](../../bindings/wasm) and is not committed.
From the repository root:

```sh
nix build            # the package, in ./result
nix flake check      # includes the tests in ./test and publint/attw
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
