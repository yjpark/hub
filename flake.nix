{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        buildInputs = with pkgs; [
          # Add extra build inputs here, etc.
          openssl
          libxkbcommon
          libGL
          wayland
          skia
          freetype
          fontconfig
        ];

        # Common derivation arguments used for all builds
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = buildInputs;
          LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

          nativeBuildInputs = with pkgs; [
            # Add extra native build inputs here, etc.
            pkg-config
            lld
          ];
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          # Additional arguments specific to this derivation can be added here.
          # Be warned that using `//` will not do a deep copy of nested
          # structures
          pname = "edger-hub-deps";
        });

        # Run clippy (and deny all warnings) on the crate source,
        # reusing the dependency artifacts (e.g. from build scripts or
        # proc-macros) from above.
        #
        # Note that this is done as a separate derivation so it
        # does not impact building just the crate by itself.
        clippy = craneLib.cargoClippy (commonArgs // {
          # Again we apply some extra arguments only to this derivation
          # and not every where else. In this case we add some clippy flags
          inherit cargoArtifacts;
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        edger-hub = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Also run the crate tests under cargo-tarpaulin so that we can keep
        # track of code coverage
        coverage = craneLib.cargoTarpaulin (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        packages.default = edger-hub;
        checks = {
         inherit
           # Build the crate as part of `nix flake check` for convenience
           edger-hub
           clippy
           coverage;
        };
      });
}
