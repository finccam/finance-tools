# `node_modules` for the npm workspace at the repository root, built offline
# from `package-lock.json`. Provides the exact versions of the JS tooling
# (TypeScript, publint, attw) that the lockfile pins.
{
  lib,
  importNpmLock,
  nodejs,
}:

importNpmLock.buildNodeModules {
  inherit nodejs;
  npmRoot = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../package.json
      ../package-lock.json
      ../packages/finance-tools/package.json
    ];
  };
}
