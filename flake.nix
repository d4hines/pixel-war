{
  inputs = {
    dream2nix.url = "github:nix-community/dream2nix";
    nixpkgs.url = "github:nixos/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    tezos.url = "github:marigold-dev/tezos-nix";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    dream2nix,
    flake-parts,
    tezos,
    rust-overlay,
    nix-filter,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux"];
      imports = [dream2nix.flakeModuleBeta];
      flake = {
        nixosModules = {
          tezos-place = import ./nix/service.nix {
            packages = self.packages;
            tezos = tezos;
          };
        };
      };
      perSystem = {
        config,
        system,
        self',
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [rust-overlay.overlays.default];
        };
        rust-toolchain = pkgs.rust-bin.stable."1.66.0".default.override {
          targets = ["wasm32-unknown-unknown"];
        };
      in {
        # define an input for dream2nix to generate outputs for
        dream2nix.inputs."rust-packages" = {
          source = nix-filter.lib {
            root = ./.;
            include = [
              "Cargo.toml"
              "Cargo.lock"
              "crates"
            ];
          };
          packageOverrides = {
            # for build-rust-package builder
            "^.*".set-toolchain.overrideRustToolchain = old: {
              cargo = rust-toolchain;
              rustc = rust-toolchain;
            };
          };
          projects = {
            rust-packages = {
              subsystem = "rust";
              translator = "cargo-lock";
            };
          };
        };
        dream2nix.inputs."frontend" = {
          source = ./frontend;
          projects = {
            frontend = {
              subsystem = "nodejs";
              translator = "yarn-lock";
            };
          };
        };
        packages = with config.dream2nix.outputs;
          rust-packages.packages
          // frontend.packages
          // {
            process-message = import ./nix/process-message.nix {
              inherit pkgs;
              tezos-packages = tezos.packages.${system};
            };
            octez-client = tezos.packages.${system}.trunk-octez-client;
          };
        devShells.default = pkgs.mkShell {
          shellHook = ''
            export CC=$(which clang)
            export PATH=/home/d4hines/repos/tezos:$PATH
          '';
          buildInputs = with pkgs;
            [
              self.packages.${system}.process-message
              # TODO: I'm going to want to fix this with dream2nix somehow
              pkg-config
              openssl

              ligo
              config.dream2nix.outputs.frontend.packages.frontend
              # config.dream2nix.outputs.kernel.devShells.kernel TODO: this doesn't work - Cargo.lock isn't found
              rustfmt
              rust-analyzer
              wabt
              clang
              # tezos.packages.${system}.trunk-octez-smart-rollup-wasm-debugger
              rust-toolchain
              zip
              unzip 

              imagemagick 
              vlc 
              ffmpeg
            ]
            ++ (with tezos.packages."${system}"; [
              # trunk-octez-smart-rollup-node-PtMumbai
              # octez-smart-rollup-client-PtMumbai
            ]);
          # TODO: this isn't working for some reason
          # But there's likely a better way. Now that I have the installer.wasm
          # file, I should be able to patch the build of the installer client
          # from Tezos
          # ++ (with config.dream2nix.outputs.rust-packages.packages; [
          #   tezos_smart_rollup_installer
          # ]);
        };
      };
    };
}
