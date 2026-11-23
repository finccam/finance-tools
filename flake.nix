{
  description = "Financial calculation primitives: a Rust crate and an npm package built from it";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1";
  };

  outputs =
    { self, nixpkgs }:
    let
      inherit (nixpkgs) lib;

      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      forEachSupportedSystem =
        f:
        lib.genAttrs supportedSystems (
          system:
          f {
            inherit system;
            pkgs = nixpkgs.legacyPackages.${system};
          }
        );
    in
    {
      packages = forEachSupportedSystem (
        { pkgs, ... }:
        rec {
          # Native build of the publishable crate; runs its own tests.
          finance-tools = pkgs.callPackage ./nix/rust-workspace.nix { };

          # Raw `wasm-bindgen` glue for `bindings/wasm` (input of the npm package).
          finance-tools-wasm = pkgs.callPackage ./nix/finance-tools-wasm.nix {
            rust-workspace = finance-tools;
          };

          node-modules = pkgs.callPackage ./nix/node-modules.nix { };

          # The npm package directory, exactly as it is packed.
          finance-tools-js = pkgs.callPackage ./nix/finance-tools-js.nix {
            inherit finance-tools-wasm node-modules;
          };

          # The npm tarball. Release CI publishes this file, so what is
          # published is byte-for-byte what `nix flake check` tested.
          finance-tools-js-tarball =
            pkgs.runCommand "${finance-tools-js.pname}-tarball-${finance-tools-js.version}"
              {
                nativeBuildInputs = [ pkgs.nodejs ];
              }
              ''
                export HOME=$TMPDIR
                cp -r --no-preserve=mode ${finance-tools-js} package
                mkdir -p $out
                npm pack --pack-destination $out ./package
              '';

          default = finance-tools-js;
        }
      );

      checks = forEachSupportedSystem (
        { pkgs, system }:
        let
          packages = self.packages.${system};
          inherit (packages)
            finance-tools
            finance-tools-js
            finance-tools-js-tarball
            node-modules
            ;

          # `RustQuant_time` (parity tests only) enables `plotters`' default
          # features, which pull in `font-kit` and with it fontconfig/freetype.
          withParityDeps = old: {
            nativeBuildInputs =
              old.nativeBuildInputs ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.pkg-config ];
            buildInputs =
              (old.buildInputs or [ ])
              ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
                pkgs.fontconfig
                pkgs.freetype
              ];
          };
        in
        {
          inherit finance-tools finance-tools-js finance-tools-js-tarball;

          # `cargo test --workspace`: RustQuant parity and the bindings' tests.
          rust-workspace-tests = finance-tools.overrideAttrs (
            old:
            withParityDeps old
            // {
              pname = "finance-tools-workspace-tests";
              cargoTestFlags = [ "--workspace" ];
              buildPhase = "runHook preBuild; runHook postBuild";
              installPhase = "touch $out";
            }
          );

          clippy = finance-tools.overrideAttrs (
            old:
            withParityDeps old
            // {
              pname = "finance-tools-clippy";
              nativeBuildInputs = (withParityDeps old).nativeBuildInputs ++ [ pkgs.clippy ];
              buildPhase = ''
                runHook preBuild
                cargo clippy --frozen --workspace --all-targets -- --deny warnings
                runHook postBuild
              '';
              doCheck = false;
              installPhase = "touch $out";
            }
          );

          # What `cargo publish` would upload, built and verified offline.
          crate-package = finance-tools.overrideAttrs {
            pname = "finance-tools-crate-package";
            buildPhase = ''
              runHook preBuild
              cargo package --frozen --package finance-tools
              runHook postBuild
            '';
            doCheck = false;
            installPhase = ''
              mkdir -p $out
              cp target/package/*.crate $out/
            '';
          };

          # `cargo fmt` needs `cargo metadata`, which needs the dependencies;
          # rustfmt on the sources directly does not.
          rustfmt = pkgs.runCommand "finance-tools-rustfmt" { nativeBuildInputs = [ pkgs.rustfmt ]; } ''
            find ${finance-tools.src} -name '*.rs' -print0 \
              | xargs -0 rustfmt --check --edition 2024
            touch $out
          '';

          # Smoke tests of the packed package, importing it by its own name.
          js-test =
            pkgs.runCommand "finance-tools-js-test"
              {
                nativeBuildInputs = [ pkgs.nodejs ];
              }
              ''
                cp -r --no-preserve=mode ${finance-tools-js} package
                cp -r ${./packages/finance-tools/test} package/test
                cd package
                node --test
                touch $out
              '';

          # Lints the tarball's manifest, exports and type resolution.
          js-package-lint =
            pkgs.runCommand "finance-tools-js-package-lint"
              {
                nativeBuildInputs = [ pkgs.nodejs ];
              }
              ''
                export HOME=$TMPDIR
                tarball=$(echo ${finance-tools-js-tarball}/*.tgz)
                ${node-modules}/node_modules/.bin/publint run --strict "$tarball"
                # ESM-only package: the CommonJS/node10 resolution modes do not apply.
                ${node-modules}/node_modules/.bin/attw --profile esm-only "$tarball"
                touch $out
              '';

          nixfmt = pkgs.runCommand "finance-tools-nixfmt" { nativeBuildInputs = [ pkgs.nixfmt ]; } ''
            nixfmt --check ${./flake.nix} ${./nix}/*.nix
            touch $out
          '';
        }
      );

      devShells = forEachSupportedSystem (
        { pkgs, system }:
        {
          default = pkgs.mkShell {
            inputsFrom = [
              self.checks.${system}.rust-workspace-tests
              self.packages.${system}.finance-tools-wasm
            ];
            packages = [
              pkgs.cargo-semver-checks
              pkgs.clippy
              pkgs.nodejs
              pkgs.rust-analyzer
              pkgs.rustfmt
            ];
            env.RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        }
      );

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt-tree);
    };
}
