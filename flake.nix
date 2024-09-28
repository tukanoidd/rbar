{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    parts,
    nci,
    ...
  }:
    parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [
        nci.flakeModule
        ./crates.nix
      ];
      perSystem = {
        pkgs,
        config,
        ...
      }: let
        crateOutputs = config.nci.outputs."rbar";
      in {
        devShells.default = crateOutputs.devShell.overrideAttrs (old: {
          packages =
            (old.packages or [])
            ++ (with pkgs; [
              cargo-edit
              cargo-expand
            ]);
          shellHook = ''
            export LIBCLANG_PATH="${pkgs.libclang.lib}/lib";
          '';
        });
        packages.default = crateOutputs.packages.release;
      };
    };
}
