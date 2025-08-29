{
  pkgs,
  flake,
  perSystem,
  ...
}:
flake.lib.devshell {
  inherit pkgs perSystem;
  rust-toolchain = pkgs.rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
}
