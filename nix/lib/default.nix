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
          cargo-llvm-cov
        ]
        ++ lib.optionals include-cyclonedds [ perSystem.self.cyclonedds-c ]
        ++ extra-packages;
    };
}
