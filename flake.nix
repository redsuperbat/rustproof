{
  description = "Rustproof - a fast, extensible code checker.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "rustproof";
        version = "0.1.2";

        src = ./.;
        cargoLock = {lockFile = ./Cargo.lock;};

        nativeBuildInputs = [pkgs.llvmPackages.libclang];

        meta = with pkgs.lib; {
          description = ''
            A fast, extensible code checker. Rustproof uses the Language Server Protocol (LSP)
            to communicate with your editor and detect spelling mistakes in your code. It handles
            a multitude of casings by breaking words into individual components.
          '';
          license = licenses.mit;
          maintainers = [maintainers.redsuperbat];
        };
      };

      devShells.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.llvmPackages.libclang
          pkgs.rust-analyzer
        ];
        DYLD_LIBRARY_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
      };
    });
}
