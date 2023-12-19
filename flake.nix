{
  description = "yaaaaaaaaaaaaaaaaaaaaa";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.11";
    nixpkgs-unstable.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs @ {self, ...}:
    inputs.flake-utils.lib.eachSystem ["x86_64-linux"] (system: let
      pkgs = import inputs.nixpkgs {
        inherit system;
      };
      unstable = import inputs.nixpkgs-unstable {
        inherit system;
      };

      manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
    in {
      # TODO: 'cargo build' won't build this
      packages.default = unstable.rustPlatform.buildRustPackage {
        pname = manifest.name;
        version = manifest.version;
        cargoLock.lockFile = ./Cargo.lock;
        src = pkgs.lib.cleanSource ./.;

        # - [nix flake rust and pkgconfig](https://discourse.nixos.org/t/nix-and-rust-how-to-use-pkgconfig/17465/3)
        buildInputs = with pkgs; [
          openssl
          glib
          gdk-pixbuf
          cairo
          pango
          atkmm
          gtk3
          webkitgtk
          libsoup
        ];
        nativeBuildInputs = with pkgs; [
          pkg-config

          nodejs_20
          unstable.bun
          cargo-tauri

          # - [RPATH, or why lld doesn't work on NixOS](https://matklad.github.io/2022/03/14/rpath-or-why-lld-doesnt-work-on-nixos.html)
          llvmPackages.bintools
          llvmPackages_15.clang

          # - [Using mold as linker prevents - NixOS Discourse](https://discourse.nixos.org/t/using-mold-as-linker-prevents-libraries-from-being-found/18530/5)
          # mold won't work without a wrapper to set correct RPATH
          # mold
          # unstable.mold
        ];
      };

      devShells.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs;
          [
            unstable.rust-analyzer
            unstable.rustfmt
            unstable.clippy

            neovim

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

            nodePackages_latest.svelte-language-server
            nodePackages_latest.typescript-language-server
            tailwindcss-language-server
          ]
          ++ self.packages."${system}".default.nativeBuildInputs
          ++ self.packages."${system}".default.buildInputs;

        RUST_BACKTRACE = 1;

        # - [could not determine accessibility bus address](https://github.com/tauri-apps/tauri/issues/4315#issuecomment-1207755694)
        WEBKIT_DISABLE_COMPOSITING_MODE = 1;
      };
    });
}
