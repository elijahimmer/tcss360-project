{
  description = "A game that is a school project";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    # Very nice to use
    flake-utils.url = "github:numtide/flake-utils";

    # Great rust build system
    naersk.url = "github:nmattia/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs = {
    self,
    flake-utils,
    naersk,
    nixpkgs,
  }: let
    supportedSystems = with flake-utils.lib.system; [
      x86_64-linux
      aarch64-linux
      aarch64-darwin
    ];
  in flake-utils.lib.eachSystem supportedSystems (system: let
    pkgs = (import nixpkgs) {
      inherit system;
    };

    naersk' = pkgs.callPackage naersk {};

    buildInputs = with pkgs;
      lib.optionals (stdenv.isLinux) [
        # for Linux
        # Audio (Linux only)
        alsa-lib-with-plugins
        # Cross Platform 3D Graphics API
        vulkan-loader
        # For debugging around vulkan
        vulkan-tools
        wayland
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        libxkbcommon
        udev 
      ];

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    all_deps = with pkgs; [
      #cargo-flamegraph
      #cargo-expand
      cmake
    ] ++ buildInputs ++ nativeBuildInputs;
  in rec {
    defaultPackage = packages.tcss360-project;
    packages.tcss360-project = naersk'.buildPackage rec {
      inherit buildInputs nativeBuildInputs;
      src = ./.;
      meta = with pkgs.lib; {
        description = "A project for TCSS 360.";
        homepage = "https://github.com/elijahimmer/tcss360-project";
        license = licenses.mit;
        mainProgram = "tcss360-project";
      };
      postInstall = ''
        #cp -r assets $out/bin/
      '';
      cargoBuildOptions = x: x ++ [ "--no-default-features" ];
    };

    devShells.default = pkgs.mkShell {
      nativeBuildInputs = all_deps;
      LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath all_deps ++ pkgs.lib.makeLibraryPath all_deps;
      shellHook = ''
        export CARGO_MANIFEST_DIR=$(pwd)
      '';
    };

    formatter = pkgs.alejandra;
  });
}
