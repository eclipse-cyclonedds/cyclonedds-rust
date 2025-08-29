{
  pkgs,
  perSystem,
  flake,
  ...
}:
flake.lib.devshell {
  inherit perSystem pkgs;
  rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile (
    builtins.toFile "nightly-toolchain.toml" (
      builtins.replaceStrings [ ''channel = "stable"'' ] [ ''channel = "nightly"'' ] (
        builtins.readFile ../../../rust-toolchain.toml
      )
    )
  );
}
