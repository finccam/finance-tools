# `bindings/wasm` compiled to `wasm32-unknown-unknown` and post-processed by
# `wasm-bindgen` (bundler target) and `wasm-opt`. The output is the raw glue
# that `packages/finance-tools` wraps; it is not a package on its own.
{
  binaryen,
  lld,
  rust-workspace,
  wasm-bindgen-cli_0_2_130,
}:

rust-workspace.overrideAttrs (old: {
  pname = "finance-tools-wasm";

  nativeBuildInputs = old.nativeBuildInputs ++ [
    binaryen
    # nixpkgs' rustc links `wasm32-unknown-unknown` with `lld` from PATH.
    lld
    # Must match the `wasm-bindgen` version pinned in `bindings/wasm/Cargo.toml`.
    wasm-bindgen-cli_0_2_130
  ];

  # `buildRustPackage`'s default build and install hooks target the host
  # platform; only its vendored dependency setup is reused here.
  buildPhase = ''
    runHook preBuild

    cargo build \
      --frozen \
      --profile release-wasm \
      --package finance-tools-wasm \
      --lib \
      --target wasm32-unknown-unknown

    wasm-bindgen \
      --target bundler \
      --out-name finance_tools \
      --out-dir pkg \
      target/wasm32-unknown-unknown/release-wasm/finance_tools_wasm.wasm

    wasm-opt -Os pkg/finance_tools_bg.wasm -o pkg/finance_tools_bg.wasm

    runHook postBuild
  '';

  # The bindings' Rust tests run natively in the `rust-workspace-tests` check;
  # the JS boundary is tested by the `js-test` check.
  doCheck = false;

  installPhase = ''
    runHook preInstall
    cp -r pkg $out
    runHook postInstall
  '';
})
