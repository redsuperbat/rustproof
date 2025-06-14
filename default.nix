{pkgs ? import <nixpkgs> {}}:
pkgs.rustPlatform.buildRustPackage {
  pname = "rustproof";
  version = "0.1.2";
  src = ./.;
  cargoLock = {lockFile = ./Cargo.lock;};
  nativeBuildInputs = [pkgs.llvmPackages.libclang];
  meta = {
    description = "A fast, extensible code checker. Rustproof uses the Language Server Protocol (LSP) to communicate with your editor and detect spelling mistakes in your code. It handles a multitude of casings by breaking words into individual components.";
    license = pkgs.lib.licenses.mit;
    maintainers = ["Max Netterberg"];
  };
}
