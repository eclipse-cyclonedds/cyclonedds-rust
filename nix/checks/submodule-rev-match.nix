{
  pkgs,
  flake,
  inputs,
  ...
}:
pkgs.runCommand "submodule-rev-match" { } ''
  if [ ! -d ${flake}/cyclonedds-sys/vendor/cyclonedds-c ]; then
    echo "SKIP: submodule not present"
    echo "  - rerun with: nix flake check .?submodules=1"
    touch $out 
  elif diff -rq ${flake}/cyclonedds-sys/vendor/cyclonedds-c ${inputs.cyclonedds-sources}; then
    echo "OK: revisions match"
    touch $out
  else
    echo ""
    echo "ERROR: submodule revision does not match revision in locked flake input"
    echo "  - Update the submodule: git submodule update --init"
    echo "  - Or update the flake input: nix flake update cyclonedds-sources"
    exit 1
  fi
''
