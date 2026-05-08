{
  description = "flake for epic majong gamejam game";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-25.11";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    fe_pkgs = fenix.packages.${system};

    librarys = with pkgs; [
      wayland
      alsa-lib-with-plugins
      udev
      wayland
      libxkbcommon
      vulkan-loader
      rustfmt
      mold
      rustc
      stdenv.cc.cc.lib
    ];
  in {
    devShells.${system}.default = pkgs.mkShell {
      buildInputs = with pkgs; [
        (fe_pkgs.complete.withComponents [
          "cargo"
          "clippy"
          "rustc"
          "rustfmt"
          "rust-analyzer"
          "rust-src"
        ])
        clang
        pkg-config
        tracy-wayland
      ] ++ librarys;

      LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath librarys}";
    };
  };
}
