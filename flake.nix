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

    buildInputs = with pkgs; [
      # makeBinaryWrapper
      pkg-config
      libxkbcommon
    ] ++ lib.optionals (lib.systems.inspect.predicates.isLinux system) [
      alsa-lib
      libudev-zero
    ];
  in {
    packages.default = naersk'.buildPackage {
      inherit buildInputs;
      src = ./.;
      meta = with pkgs.lib; {
        description = "A project for TCSS 360.";
        homepage = "https://github.com/elijahimmer/tcss360-project";
        license = licenses.mit;
        mainProgram = "tcss360-project";
      };

      /*postInstall = ''
        wrapProgram $out/bin/wlrs-bar \
      '';*/
    };

    devShells.default = pkgs.mkShell {
      buildInputs =
        buildInputs
        ++ (with pkgs; [
          cargo
          rustc
          clippy
        ]);
    };

    formatter = pkgs.alejandra;
  });
}
