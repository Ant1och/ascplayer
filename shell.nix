{
  pkgs ? import <nixpkgs> { },
}:

with pkgs;

mkShell rec {
  shellHook = '' 
    export PATH="/home/$USER/.cargo/bin:$PATH"
  '';

  packages = with pkgs; [
    (pkgs.fenix.complete.withComponents [
      "cargo"
      "clippy"
      "rust-src"
      "rustc"
      "rustfmt"
      "llvm-tools-preview"
      "rustc-codegen-cranelift-preview"
    ])
    rust-analyzer-nightly
    cargo-llvm-cov
    cargo-nextest
    cargo-mutants
    cargo-watch
    cargo-audit
    cargo-deny
    grcov
  ];

  nativeBuildInputs = [
    clang_19
    pkg-config
  ];

  buildInputs = [
    udev
    alsa-lib
    alsa-utils
    alsa-oss
    ffmpeg
    ffmpeg.dev
    libclang
    libclang.dev
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  PKG_CONFIG_PATH = "${pkgs.alsa-lib.dev}/lib/pkgconfig";

  # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
