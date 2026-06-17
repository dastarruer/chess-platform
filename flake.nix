{
  description = "A Nix-flake-based Rust development environment";

  nixConfig = {
    extra-substituters = [
      "https://fenix.cachix.org"
    ];
    extra-trusted-public-keys = [
      "fenix.cachix.org-1:ecJhr+RdYEdcVgUkjruiYhjbBloIEGov7bos90cZi0Q="
    ];
  };

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {self, ...} @ inputs: let
    supportedSystems = [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ];
    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (
        system: let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [
              inputs.self.overlays.default
            ];
          };
        in
          f {
            inherit system pkgs;
            preCommitCheck = inputs.git-hooks.lib.${system}.run {
              src = ./.;
              hooks = {
                alejandra.enable = true;

                clippy = {
                  enable = true;
                  packageOverrides = {
                    cargo = pkgs.rustToolchain;
                    clippy = pkgs.rustToolchain;
                  };
                  settings = {
                    allFeatures = true;
                    denyWarnings = true;
                  };
                };

                rustfmt = {
                  enable = true;
                  packageOverrides = {
                    cargo = pkgs.rustToolchain;
                    rustfmt = pkgs.rustToolchain;
                  };
                  settings.check = true;
                };

                check-toml.enable = true;
                taplo.enable = true;

                prettier = {
                  enable = true;
                  settings.configPath = ".prettierrc";
                };
                markdownlint.enable = true;
              };
            };
          }
      );
  in {
    overlays.default = final: prev: {
      rustToolchain = inputs.fenix.packages.${prev.stdenv.hostPlatform.system}.fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-mvUGEOHYJpn3ikC5hckneuGixaC+yGrkMM/liDIDgoU=";
      };
    };

    devShells = forEachSupportedSystem (
      {
        pkgs,
        system,
        preCommitCheck,
        ...
      }: {
        default = pkgs.mkShell {
          # Install pre-commit file
          shellHook = preCommitCheck.shellHook;

          packages = with pkgs; [
            # RUST
            rustToolchain
            openssl
            pkg-config
            rust-analyzer

            # NIX
            self.formatter.${system}
            nixd
          ];

          env = {
            # Required by rust-analyzer
            RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";
          };
        };
      }
    );

    formatter = forEachSupportedSystem ({pkgs, ...}: pkgs.alejandra);
  };
}
