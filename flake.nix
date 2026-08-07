{
  description = "OpenE2E";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        cargoManifest = builtins.fromTOML (builtins.readFile ./Cargo.toml);
        lib = pkgs.lib;

        # GUI-specific runtime dependencies
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

        # CLI has no extra runtime dependencies
        cliLibraries = [ ];

        # Build a variant (CLI or GUI)
        mkOpenE2E = { name, features, libraries }:
          pkgs.rustPlatform.buildRustPackage {
            pname = cargoManifest.package.name;
            version = cargoManifest.package.version;
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;

            doCheck = false;
            cargoBuildFlags = lib.optionals (features != "") [ "--features" features ];

            buildInputs = libraries;
            nativeBuildInputs = with pkgs; [
              pkg-config
              cmake
              makeWrapper
            ];

            postFixup = ''
              ${lib.optionalString (libraries != []) ''
                mkdir -p $out/lib

                # Get library paths and split by colon
                libPath="${pkgs.lib.makeLibraryPath libraries}"

                # Copy all .so* files to $out/lib
                for libdir in $(echo "$libPath" | tr ':' '\n'); do
                  if [ -d "$libdir" ]; then
                    cp -v "$libdir"/*.so* $out/lib/ 2>/dev/null || true
                  fi
                done
              ''}

              # Rename the binary
              mv $out/bin/${cargoManifest.package.name} $out/bin/.${cargoManifest.package.name}-wrapped

              # Create wrapper
              cat > $out/bin/${cargoManifest.package.name} << 'WRAPPER'
              #!/usr/bin/env bash
              SCRIPT_DIR="$(cd "$(dirname "''${BASH_SOURCE[0]}")" && pwd)"
              ${lib.optionalString (libraries != []) ''export LD_LIBRARY_PATH="$SCRIPT_DIR/../lib''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"''}
              exec "$SCRIPT_DIR/.${cargoManifest.package.name}-wrapped" "$@"
              WRAPPER

              chmod +x $out/bin/${cargoManifest.package.name}
            '';

            meta = with pkgs.lib; {
              description = "OpenE2E";
              platforms = platforms.linux;
            };
          };

        # Build both variants
        cliPackage = mkOpenE2E { name = "cli"; features = ""; libraries = cliLibraries; };
        guiPackage = mkOpenE2E { name = "gui"; features = "gui"; libraries = guiLibraries; };

      in {
        packages = {
          default = cliPackage;
          cli = cliPackage;
          gui = guiPackage;
        };

        apps = {
          default = {
            type = "app";
            program = "${cliPackage}/bin/${cargoManifest.package.name}";
          };
          cli = {
            type = "app";
            program = "${cliPackage}/bin/${cargoManifest.package.name}";
          };
          gui = {
            type = "app";
            program = "${guiPackage}/bin/${cargoManifest.package.name}";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            (rust-bin.stable.latest.default.override {
              extensions = [ "rust-src" ];
            })

            # Build tools
            cargo
            rustc
            rust-analyzer
            clippy

            # Development tools
            pkg-config
            cmake
            slint-lsp

            # All runtime dependencies (for dev environment)
          ] ++ guiLibraries;

          shellHook = ''
            export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath guiLibraries}:$LD_LIBRARY_PATH"
          '';
        };
      }
    );
}
