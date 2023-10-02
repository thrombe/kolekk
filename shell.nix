{ nixpkgs ? import <nixpkgs> {}, nixpkgs-unstable ? import <nixos-unstable> {} }:

let
  unstable-overlays = self: super: {
    # bun 1.0 is not available in nixpkgs yet
    bun = super.bun.overrideAttrs (old: rec {
      version = "1.0.0";
      src = pkgs.fetchurl {
        url = "https://github.com/oven-sh/bun/releases/download/bun-v${version}/bun-linux-x64.zip";
        hash = "sha256-1ju7ZuW82wRfXEiU24Lx9spCoIhhddJ2p4dTTQmsa7A=";
      };
    });
  };
  stable-overlays = self: super: {
    
  };

  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "2ab91c8d65c00fd22a441c69bbf1bc9b420d5ea1";
    sha256 = "sha256-wrsPjsIx2767909MPGhSIOmkpGELM9eufqLQOPxmZQg";
  };
  unstablePinned = nixpkgs.fetchFromGitHub {
    owner = "NixOS";
    repo = "nixpkgs";
    rev = "e7f38be3775bab9659575f192ece011c033655f0";
    sha256 = "sha256-vYGY9bnqEeIncNarDZYhm6KdLKgXMS+HA2mTRaWEc80";
  };
  pkgs = import pinnedPkgs { overlays = [ stable-overlays ]; };
  unstable = import unstablePinned { overlays = [ unstable-overlays ]; };
in pkgs.mkShell {
    packages = with pkgs; [
        # rustup
        unstable.cargo
        unstable.rustc
        unstable.clippy
        # - [RPATH, or why lld doesn't work on NixOS](https://matklad.github.io/2022/03/14/rpath-or-why-lld-doesnt-work-on-nixos.html)
        llvmPackages.bintools
        # llvmPackages_15.bintools
        # llvmPackages_15.llvm
        llvmPackages_15.clang

        
        # - [Using mold as linker prevents - NixOS Discourse](https://discourse.nixos.org/t/using-mold-as-linker-prevents-libraries-from-being-found/18530/5)
        # mold won't work without a wrapper to set correct RPATH
        # mold
        # unstable.mold

        cargo-tauri
        pkg-config

        openssl
        glib
        gdk-pixbuf
        cairo
        pango
        atkmm
        gtk3
        webkitgtk
        libsoup
        # libxml2

        # musiplayer
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        # gst_all_1.gst-plugins-good
        gst_all_1.gst-plugins-bad
        # gst_all_1.gst-plugins-ugly
        # gst_all_1.gst-plugins-rs

        nodejs_20
        unstable.bun

        unstable.rust-analyzer
        nodePackages_latest.svelte-language-server
        nodePackages_latest.typescript-language-server

        neovim
    ];

    # RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    # LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
    # Add precompiled library to rustc search path
    # RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
    #   # add libraries here (e.g. pkgs.libvmi)
    # ]);
    # Add glibc, clang, glib and other headers to bindgen search path
    # BINDGEN_EXTRA_CLANG_ARGS = 
    # # Includes with normal include path
    # (builtins.map (a: ''-I"${a}/include"'') [
    #   # add dev libraries here (e.g. pkgs.libvmi.dev)
    #   pkgs.glibc.dev 
    # ])
    # # Includes with special directory paths
    # ++ [
    #   ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
    #   ''-I"${pkgs.glib.dev}/include/glib-2.0"''
    #   ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
    # ];

    shellHook = ''
        # export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
        # export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
        # SHELL=$(which zsh)
        # zsh -ic "ze kolekk"
        # zellij -l kolekk
        # exit
    '';
}