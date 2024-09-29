{...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: let
    crateName = "rbar";
  in {
    nci = {
      projects = {
        "rbar" = {
          path = ./.;
          export = false;
        };
      };
      crates = {
        ${crateName} = {
          export = true;
          runtimeLibs = with pkgs; [
            wayland
            vulkan-loader
          ];
          depsDrvConfig = {
            mkDerivation = {
              nativeBuildInputs = with pkgs; [
                pkg-config
                libxkbcommon
              ];
            };
          };
        };
      };
    };
  };
}
