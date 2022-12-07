{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) lib;
  l = nixpkgs.lib // builtins;

  name = "jormungandr";
  root = inputs.self + "/src/${name}";

  mkSimplePkg = subPkg: lib.mkPackage {pkgPath = root + "/${subPkg}";};
in {
  jormungandr = lib.mkPackage {
    pkgPath = root + "/jormungandr";
    cargoOptions = [
      "--features"
      "prometheus-metrics"
    ];
  };
  jcli = mkSimplePkg "jcli";
}
