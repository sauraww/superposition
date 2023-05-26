{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        naersk' = pkgs.callPackage naersk {};

      in rec {
        # For `nix build` & `nix run`:
        defaultPackage = naersk'.buildPackage {
          nativeBuildInputs = with pkgs; [ postgresql_12 ];
          src = ./.;
        };

        # For `nix develop`:
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs;
          [
            # Build requirements
            rustc
            cargo
            libiconv
            openssl
            postgresql_12
            # Extras
            rust-analyzer
            rustfmt
            bacon
            cargo-watch
            diesel-cli
            docker-compose
          ];
          # buildInputs = with pkgs; [ ];
        };
      }
    );
}