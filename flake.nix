{
  description = "Themeforge Helix theme semantic converter";

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
          common = {
            pname = "themeforge";
            version = "0.1.0";
            src = builtins.path {
              path = ./.;
              name = "themeforge-source";
            };
            cargoHash = "sha256-r2pPZoTcOHjZKibpZ5LTflN02BfvhDnk6a6TTH5rluk=";
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
              pname = "themeforge-tests";
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
            ];

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };
        }
      );
    };
}
