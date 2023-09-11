{ nixpkgs ? import <nixpkgs> {}, unstable ? import <nixos-unstable> {} }:

let
  pinnedPkgs = nixpkgs.fetchFromGitHub {
    owner  = "NixOS";
    repo   = "nixpkgs";
    rev    = "2ab91c8d65c00fd22a441c69bbf1bc9b420d5ea1";
    sha256 = "sha256-wrsPjsIx2767909MPGhSIOmkpGELM9eufqLQOPxmZQg";
  };
  pkgs = import pinnedPkgs {};
in pkgs.mkShell {
    packages = with pkgs; [
        rustup
        llvmPackages_15.clang

        cargo-tauri
        openssl
        pkg-config
        glib
        gdk-pixbuf
        cairo
        pango
        atkmm
        gtk3
        webkitgtk
        libsoup

        # musiplayer
        gst_all_1.gstreamer
        gst_all_1.gst-plugins-base
        gst_all_1.gst-plugins-bad

        nodejs_20
    ];
}
