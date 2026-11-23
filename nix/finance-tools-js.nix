# The `finance-tools` npm package, laid out exactly as it is packed: the
# TypeScript API in `packages/finance-tools/src` compiled against the
# `wasm-bindgen` glue from `finance-tools-wasm`.
{
  lib,
  stdenvNoCC,
  finance-tools-wasm,
  node-modules,
  nodejs,
}:

let
  packageJson = lib.importJSON ../packages/finance-tools/package.json;
in
stdenvNoCC.mkDerivation {
  pname = "${packageJson.name}-js";
  inherit (packageJson) version;

  src = lib.fileset.toSource {
    root = ../packages/finance-tools;
    fileset = lib.fileset.unions [
      ../packages/finance-tools/package.json
      ../packages/finance-tools/README.md
      ../packages/finance-tools/tsconfig.json
      ../packages/finance-tools/src
    ];
  };

  nativeBuildInputs = [ nodejs ];

  buildPhase = ''
    runHook preBuild

    cp -r --no-preserve=mode ${finance-tools-wasm} src/wasm
    ${node-modules}/node_modules/.bin/tsc -p tsconfig.json
    cp -r src/wasm dist/wasm

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall

    mkdir -p $out
    cp -r package.json README.md dist $out/
    cp ${../LICENSE-MIT} $out/LICENSE-MIT
    cp ${../LICENSE-APACHE} $out/LICENSE-APACHE

    runHook postInstall
  '';

  meta = {
    inherit (packageJson) description;
    homepage = "https://github.com/finccam/finance-tools";
    license = with lib.licenses; [
      mit
      asl20
    ];
    platforms = lib.platforms.all;
  };
}
