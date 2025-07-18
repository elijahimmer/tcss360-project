{
  description = "A game that is a school project";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    # Very nice to use
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = {
    self,
    flake-utils,
    nixpkgs,
  }: let
    supportedSystems = with flake-utils.lib.system; [
      x86_64-linux
      aarch64-linux
      aarch64-darwin
    ];
  in
    flake-utils.lib.eachSystem supportedSystems (system: let
      pkgs = (import nixpkgs) {
        inherit system;
      };
    in rec {
      devShells.default = pkgs.mkShell rec {
        buildInputs = with pkgs;
          pkgs.lib.optionals (stdenv.isLinux) [
            # for Linux
            # Audio (Linux only)
            alsa-lib-with-plugins
            # Cross Platform 3D Graphics API
            libxkbcommon
            udev
            vulkan-loader
            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
            wayland
          ];
        nativeBuildInputs = with pkgs; [
          pkg-config
          # For debugging around vulkan
          vulkan-tools
          sqlite
          lld
          rustc
          cargo
          just

          # Wasm
          wasm-bindgen-cli_0_2_100
          trunk
          binaryen
        ];
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
      };

      defaultPackage = packages.a-hex-befalls;
      packages.a-hex-befalls = pkgs.callPackage ./package.nix {};

      formatter = pkgs.alejandra;
    });
}
