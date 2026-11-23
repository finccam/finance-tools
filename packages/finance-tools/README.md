# finance-tools

Day count conventions for JavaScript, powered by the
[`finance-tools`](https://crates.io/crates/finance-tools) Rust crate compiled
to WebAssembly.

```js
import { DayCountConvention, conventionName, yearFraction } from "finance-tools";

yearFraction(DayCountConvention.Actual365Fixed, "2024-01-01", "2024-07-01"); // 0.4986…
conventionName(DayCountConvention.ThirtyE360); // "30E/360"
```

Dates are ISO 8601 calendar dates (`YYYY-MM-DD`). JavaScript `Date` objects
are not accepted, because they are instants and turning one into a calendar
date depends on a time zone. `yearFraction` throws on invalid input.

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
