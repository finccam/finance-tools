# The Cargo workspace at the repository root, built natively. The default
# build covers only the publishable `finance-tools` crate and its own tests,
# which need no system libraries; `flake.nix` derives the workspace-wide
# checks (RustQuant parity, clippy) from this with `overrideAttrs`.
{
  lib,
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
  cargoTestFlags = [
    "--package"
    "finance-tools"
  ];

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
