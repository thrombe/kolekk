{ pkgs ? import <nixpkgs> {}, unstable ? import <nixos-unstable> {} }:

pkgs.mkShell {
    packages = with pkgs; [
        # rustup
        unstable.cargo
        unstable.rustc
        unstable.clippy
        # - [RPATH, or why lld doesn't work on NixOS](https://matklad.github.io/2022/03/14/rpath-or-why-lld-doesnt-work-on-nixos.html)
        llvmPackages.bintools
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

        # musiplayer
        mpv
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-good
        gst_all_1.gst-plugins-bad
        # gst_all_1.gst-plugins-ugly
        # Plugins to reuse ffmpeg to play almost every video format
        # gst_all_1.gst-libav
        # Support the Video Audio (Hardware) Acceleration API
        # gst_all_1.gst-vaapi

        nodejs_20
        unstable.bun

        unstable.rust-analyzer
        # nodePackages_latest.svelte-language-server
        # nodePackages_latest.typescript-language-server

        neovim
    ];
}
