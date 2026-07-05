{
  description = "Helix Theme Transformer";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              let
                base = baseNameOf path;
              in
              !(
                base == ".direnv"
                || base == ".git"
                || base == "target"
                || base == "result"
                || pkgs.lib.hasPrefix "result-" base
              );
          };
          common = {
            pname = "helix-theme-transformer";
            version = "0.1.0";
            inherit src;
            cargoHash = "sha256-RxYOFPmPNrAa4bU5w7RJJqc6SY2hrKN9gxo6sPQEFGs=";
            nativeBuildInputs = [
              pkgs.pkg-config
            ];
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage (
            common
            // {
              doCheck = false;
            }
          );

          tests = pkgs.rustPlatform.buildRustPackage (
            common
            // {
              pname = "helix-theme-transformer-tests";
              doCheck = true;
              cargoTestFlags = [
                "--workspace"
                "--all-targets"
                "--all-features"
              ];
              installPhase = ''
                runHook preInstall
                mkdir -p "$out"
                cp -R . "$out/checkout"
                runHook postInstall
              '';
            }
          );
        }
      );

      checks = forAllSystems (system: {
        default = self.packages.${system}.tests;
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
        in
        {
          default = pkgs.mkShell {
            packages = [
              pkgs.cargo
              pkgs.clippy
              pkgs.rust-analyzer
              pkgs.rustc
              pkgs.rustfmt
              pkgs.unzip
              pkgs.zip
            ];

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };
        }
      );
    };
}
