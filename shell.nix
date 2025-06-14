{pkgs ? import <nixpkgs> {}}: let
  libclang = pkgs.llvmPackages.libclang;
in
  pkgs.mkShell {
    buildInputs = [libclang];
    DYLD_LIBRARY_PATH = "${libclang.lib}/lib";
  }
