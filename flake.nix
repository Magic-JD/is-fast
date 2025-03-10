{
  description = "is-fast";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs =
    { self, nixpkgs, ... }:
    let
      forAllSystems =
        function:
        nixpkgs.lib.genAttrs [
          "x86_64-linux"
          "aarch64-linux"
          "x86_64-darwin"
          "aarch64-darwin"
        ] (system: function nixpkgs.legacyPackages.${system});

      darwinDeps =
        pkgs: with pkgs; [
          darwin.apple_sdk.frameworks.SystemConfiguration
          libiconv
        ];
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          name = "is-fast";
          packages =
            (with pkgs; [
              cargo
              cargo-edit
              clippy
              rustc
            ])
            ++ (pkgs.lib.optional pkgs.stdenvNoCC.isDarwin (darwinDeps pkgs));
        };
      });
      formatter = forAllSystems (pkgs: pkgs.nixfmt-rfc-style);
      packages = forAllSystems (pkgs: {
        is-fast =
          with pkgs;
          let
            fs = lib.fileset;
            sourceFiles = fs.unions [
              ./Cargo.lock
              ./Cargo.toml
              ./src
            ];

            cargoToml = with builtins; (fromTOML (readFile ./Cargo.toml));
            pname = cargoToml.package.name;
            version = cargoToml.package.version;
            cargoLock.lockFile = ./Cargo.lock;
            darwinBuildInputs = (darwinDeps pkgs);
          in
          pkgs.rustPlatform.buildRustPackage {
            inherit pname version cargoLock;
            src = fs.toSource {
              root = ./.;
              fileset = sourceFiles;
            };
            nativeBuildInputs = [
              clippy
              rustfmt
              openssl
            ];
            buildInputs = [ ] ++ lib.optionals stdenv.isDarwin darwinBuildInputs;

            app_test = ''
              cargo fmt --manifest-path ./Cargo.toml --all --check
              cargo clippy -- --deny warnings
              cargo test --verbose
            '';

            preBuildPhases = [ "app_test" ];

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
