{
  description = "is-fast";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, ... }:
    let
      forAllSystems = f:
        nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ] (system: f nixpkgs.legacyPackages.${system});

      darwinDeps = pkgs: with pkgs; [
        darwin.apple_sdk.frameworks.SystemConfiguration
        libiconv
      ];
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "is-fast";
          packages = with pkgs; [
            cargo
            cargo-edit
            clippy
            rustc
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (darwinDeps pkgs);
        };
      });

      packages = forAllSystems (pkgs: {
        is-fast =
          with pkgs;
          let
            cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            cargoLock = { lockFile = ./Cargo.lock; };
            darwinBuildInputs = darwinDeps pkgs;
          in
          pkgs.rustPlatform.buildRustPackage {
            inherit pname version cargoLock;
            src = ./.;
            nativeBuildInputs = [
              clippy
              rustfmt
              openssl
            ];
            buildInputs = pkgs.lib.optionals pkgs.stdenv.isDarwin darwinBuildInputs;

            preBuild = ''
              cargo fmt --manifest-path ./Cargo.toml --all --check
              cargo clippy -- --deny warnings
              cargo test -- --skip=generate_config::tests::test_run_creates_config_file
            '';
          };

        default = self.packages.${pkgs.system}.is-fast;
      });

      apps = forAllSystems (pkgs: {
        default = {
          type = "app";
          program = "${self.packages.${pkgs.system}.is-fast}/bin/is-fast";
        };
      });
    };
}
