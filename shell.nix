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
    lld

    glib
    glibc
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
    vulkan-loader
    libxkbcommon
    wayland
    ffmpeg
    ffmpeg.dev
    libclang
    libclang.dev
    openssl
    openssl.dev
    libz
  
    glib
    glibc
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
  PKG_CONFIG_PATH = "${pkgs.alsa-lib.dev}/lib/pkgconfig";

  # Certain Rust tools won't work without this
  # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
  # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
# with pkgs;

# mkShell {
#   nativeBuildInputs = with pkgs; [ llvm clang_19 pkg-config rustc rustup cargo gcc rustfmt ffmpeg ffmpeg.dev clippy ];
#   BuildInputs = with pkgs; [ alsa-lib.dev alsa-lib alsa-oss alsa-utils ];

#   LD_LIBRARY_PATH = with pkgs; lib.makeLibraryPath [
#     alsa-lib
#     alsa-lib.dev
#     alsa-utils
#     alsa-oss
#     ffmpeg.dev
#     libclang
#     libclang.dev
#   ];

#   PKG_CONFIG_PATH = "${pkgs.alsa-lib.dev}/lib/pkgconfig";

#   # Certain Rust tools won't work without this
#   # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
#   # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
#   RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
# }
