# finance-tools

Financial calculation primitives, written in Rust and also shipped to
JavaScript via WebAssembly.

| Path                           | What                                                   | Published as          |
| ------------------------------ | ------------------------------------------------------ | --------------------- |
| `crates/finance-tools`         | The core crate: day count conventions on `jiff` dates. | crates.io             |
| `crates/finance-tools-parity`  | Parity tests against `RustQuant_time`.                 | —                     |
| `bindings/wasm`                | `wasm-bindgen` bindings for the core crate.            | — (build input)       |
| `packages/finance-tools`       | The npm package: a TypeScript API over the bindings.   | npm                   |
| `nix/`                         | Derivations used by `flake.nix`.                       | —                     |

The layering is deliberate:

- The **core crate** is plain Rust with no target-specific dependencies or
  features, so any Rust project can depend on it.
- **Bindings** for a host language live in their own unpublished crate. They
  own everything specific to that host (the exact `wasm-bindgen` pin, global
  wasm exports, the size-optimised `release-wasm` profile).
- The **npm package** owns its public API (`src/index.ts`) and its version.
  The generated glue is an internal detail of it.

## Building

Everything is built and checked with Nix (flakes):

```sh
nix build                                 # the npm package (default), in ./result
nix build .#finance-tools-js-tarball      # the npm tarball, as it would be published
nix build .#finance-tools                 # native build of the crate, runs its tests
nix flake check                           # everything below
nix develop                               # shell with cargo, clippy, wasm-bindgen, node, …
nix fmt                                   # format Nix files
```

`nix flake check` runs the crate's tests, the RustQuant parity tests, clippy
(pedantic), rustfmt, `cargo package` for the crate, the JS tests against the
packed package, `publint` and `attw` on the npm tarball, and nixfmt. CI also
tests the crate on its declared `rust-version`.

Rust tests run under [cargo-nextest](https://nexte.st). Inside `nix develop`:

```sh
cargo nextest run --workspace     # unit, integration and parity tests
cargo test --workspace --doc      # doctests (nextest does not run them)
npm ci                            # JS tooling
```

## Releasing

The crate and the npm package are versioned independently. Publishing is done
by hand from a clean checkout of `main` after `nix flake check` passes:

```sh
# crate: bump `version` in crates/finance-tools/Cargo.toml (and the matching
# `finance-tools` entry in the root Cargo.toml), then
cargo publish --package finance-tools
git tag finance-tools-v<version>

# npm: bump `version` in packages/finance-tools/package.json, then
nix build .#finance-tools-js-tarball
npm publish result/*.tgz
git tag npm-finance-tools-v<version>
```

Publishing the tarball rather than running `npm publish` in the package
directory ships exactly what `nix flake check` tested. Publish the crate first
when an npm release depends on its changes. Tags are namespaced per artifact
because the two are versioned independently.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
