{
  description = "OpenE2E";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    # Rust toolchain overlay for access to latest Rust versions
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      # Use the same nixpkgs version to avoid duplication
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    # Generate outputs for all default systems (x86_64-linux, aarch64-linux, etc.)
    flake-utils.lib.eachDefaultSystem (system:
      let
        # Apply rust-overlay to nixpkgs to enable Rust tooling
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        lib = pkgs.lib;

        version = "1.3.0";

        # Dependencies for GUI build
        guiLibraries = with pkgs; [
          fontconfig
          freetype
          libxkbcommon
          wayland
          libffi
          bzip2
          libpng
          brotli
          zlib
        ];

        # Function to build CLI and GUI variants with different features and dependencies
        mkPackage = { features, libraries }:
          pkgs.rustPlatform.buildRustPackage {
            pname = "OpenE2E";
            inherit version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            doCheck = false;

            # Pass feature flags to cargo
            cargoBuildFlags = lib.optionals (features != "") [ "--features" features ];
            buildInputs = libraries;
            nativeBuildInputs = with pkgs; [ pkg-config cmake ];

            # Bundle shared libraries into output and create wrapper to set LD_LIBRARY_PATH
            # This ensures bundled binaries can find their dependencies at runtime
            postFixup = lib.optionalString (libraries != []) ''
              mkdir -p $out/lib
              # Extract library paths and copy .so files into bundled lib directory
              for libdir in $(echo "${pkgs.lib.makeLibraryPath libraries}" | tr ':' '\n'); do
                cp -v "$libdir"/*.so* $out/lib/ 2>/dev/null || true
              done
              # Rename actual binary and create shell wrapper to inject lib path
              mv $out/bin/OpenE2E $out/bin/.OpenE2E-wrapped
              cat > $out/bin/OpenE2E << 'WRAPPER'
              #!/usr/bin/env bash
              # Prepend bundled lib directory to library search path
              export LD_LIBRARY_PATH="$(cd "$(dirname "$0")" && pwd)/../lib:$LD_LIBRARY_PATH"
              exec "$(dirname "$0")/.OpenE2E-wrapped" "$@"
              WRAPPER
              chmod +x $out/bin/OpenE2E
            '';
          };

      in {
        packages = {
          default = self.packages.${system}.cli;
          # CLI package with no GUI features or dependencies
          cli = mkPackage { features = ""; libraries = []; };
          # GUI package with feature flag and graphical libraries bundled
          gui = mkPackage { features = "gui"; libraries = guiLibraries; };
        };

        apps = {
          default = self.apps.${system}.cli;
          cli = {
            type = "app";
            program = "${self.packages.${system}.cli}/bin/OpenE2E";
          };
          gui = {
            type = "app";
            program = "${self.packages.${system}.gui}/bin/OpenE2E";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" "rust-analyzer" "clippy" ];
            })
            pkg-config
            cmake
            slint-lsp
          ] ++ guiLibraries;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath guiLibraries}:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
}
