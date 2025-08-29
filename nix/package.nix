{
  pkgs,
  inputs,
  perSystem,
  ...
}:
let
  src = ../.;
  rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile "${src}/rust-toolchain.toml";
  rust-platform = pkgs.makeRustPlatform {
    cargo = rust-toolchain;
    rustc = rust-toolchain;
  };
  crane-lib = (inputs.crane.mkLib pkgs).overrideToolchain rust-toolchain;
  common-args = {
    buildInputs =
      with pkgs;
      [
        perSystem.self.cyclonedds-c
        rust-platform.bindgenHook
      ]
      ++ (lib.optionals stdenv.isDarwin [ libiconv ]);
    src =
      let
        fs = pkgs.lib.fileset;
      in
      fs.toSource {
        root = src;
        fileset = fs.unions [
          (fs.fileFilter (file: file.hasExt "h") src)
          (fs.fromSource (crane-lib.cleanCargoSource src))
        ];
      };
    strictDeps = true;
  };
in
crane-lib.buildPackage (
  common-args
  // {
    cargoArtifacts = crane-lib.buildDepsOnly common-args;
  }
)
