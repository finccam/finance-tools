# The Cargo workspace at the repository root, built natively. Builds the
# publishable `finance-tools` crate and runs the whole workspace's tests,
# including the property tests against QuantLib.
{
  lib,
  boost,
  quantlib,
  rustPlatform,
}:

let
  cargoToml = lib.importTOML ../crates/finance-tools/Cargo.toml;
  workspaceToml = lib.importTOML ../Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  inherit (cargoToml.package) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../clippy.toml
      ../LICENSE-MIT
      ../LICENSE-APACHE
      ../bindings/wasm
      ../crates
    ];
  };

  cargoLock.lockFile = ../Cargo.lock;

  cargoBuildFlags = [
    "--package"
    "finance-tools"
  ];
  cargoTestFlags = [ "--workspace" ];

  # Only the tests need QuantLib: `quantlib-ffi` (a dev-dependency) compiles a
  # C++ shim against it.
  checkInputs = [
    boost
    quantlib
  ];

  # Tests run under cargo-nextest, which does not run doctests; those run
  # separately with the same package selection.
  useNextest = true;
  postCheck = ''
    cargo test --frozen --doc $cargoTestFlags
  '';

  meta = {
    inherit (cargoToml.package) description;
    homepage = workspaceToml.workspace.package.repository;
    license = with lib.licenses; [
      mit
      asl20
    ];
    platforms = lib.platforms.all;
  };
}
