{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell rec {
  buildInputs = with pkgs; [
    clang_17
    llvmPackages_17.bintools
    rustup
    cargo-binutils
    cargo-espflash
    cargo-leptos
    cargo-sort
    probe-rs
    sass
    tailwindcss
    just
    (python311.withPackages (ps: with ps; [ pyserial ]))
  ];
  RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
  LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_17.libclang.lib ];
  shellHook = ''
    export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
    export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
  '';
  #RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
  #]);
  BINDGEN_EXTRA_CLANG_ARGS = (builtins.map (a: ''-I"${a}/include"'') [ pkgs.glibc.dev ]) ++ [
    ''-I"${pkgs.llvmPackages_17.libclang.lib}/lib/clang/${pkgs.llvmPackages_17.libclang.version}/include"''
    ''-I"${pkgs.glib.dev}/include/glib-2.0"''
    ''-I"${pkgs.glib.out}/lib/glib-2.0/include/"''
  ];
  PKG_CONFIG_PATH = "${pkgs.udev.dev}/lib/pkgconfig";
}
