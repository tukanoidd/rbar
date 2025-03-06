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
          runtimeLibs = with pkgs;
          with xorg; [
            wayland
            vulkan-loader
            libX11
          ];

          drvConfig = {
            mkDerivation = {
              nativeBuildInputs = with pkgs; [
                pkg-config

                libxkbcommon
                glib.dev
                pipewire.dev
              ];
            };
          };
        };
      };
    };
  };
}
