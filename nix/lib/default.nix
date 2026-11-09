{
  inputs,
  ...
}:
{
  devshell =
    {
      pkgs,
      perSystem,
      include-cyclonedds ? true,
      extra-packages ? [ ],
      rust-toolchain,
    }:
    let
      crane-lib = (inputs.crane.mkLib pkgs).overrideToolchain rust-toolchain;
    in
    crane-lib.devShell {
      LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
      inputsFrom = pkgs.lib.optionals include-cyclonedds [
        perSystem.self.default
      ];

      packages =
        with pkgs;
        [
          cargo-audit
          cargo-deny
          cargo-nextest
          # TODO: pin back to stable nix once it's >=0.9.0.
          # This bump to unstable was needed to fix coverage reports on the new
          # v2 version of cargos build layouts.
          perSystem.nixpkgs-unstable.cargo-llvm-cov
        ]
        ++ lib.optionals include-cyclonedds [ perSystem.self.cyclonedds-c ]
        ++ extra-packages;
    };
}
