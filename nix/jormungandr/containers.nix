{
  inputs,
  cell,
}: let
  inherit (inputs) nixpkgs std;
  inherit (inputs.cells.lib) constants lib;
  l = nixpkgs.lib // builtins;

  mkOCI = namespace: let
    # TODO: fix git rev
    # rev =
    #   if (inputs.self ? rev)
    #   then inputs.self.rev
    #   else "dirty";
    image = std.lib.ops.mkStandardOCI {
      name = "${constants.registry}/jormungandr";
      #tag = "${rev}-${namespace}";
      operable = cell.operables."jormungandr-${namespace}";
      debug = true;
    };
  in
    image
    // {
      imageTag = let
        hash = l.head (l.strings.splitString "-" (baseNameOf image.outPath));
      in "${hash}-${namespace}";
    };
in
  {}
  // lib.mapToNamespaces "jormungandr" mkOCI
