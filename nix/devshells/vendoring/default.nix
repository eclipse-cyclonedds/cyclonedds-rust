{
  pkgs,
  flake,
  perSystem,
  ...
}:
flake.lib.devshell {
  inherit pkgs perSystem;

  include-cyclonedds = false;
  rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ../../../rust-toolchain.toml;
}
