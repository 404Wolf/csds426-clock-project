{
  description = "CSDS426 Clock Project";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    zmap.url = "github:zmap/zmap";
    zmap.flake = false;
    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      zmap,
      ...
    }@inputs:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        rustToolchain = inputs.fenix.packages.${system}.stable.withComponents [
          "cargo"
          "clippy"
          "rustc"
          "rustfmt"
          "rust-src"
          "rust-analyzer"
        ];

        zmapPackage = pkgs.callPackage ./icmp_clocksync/build.nix { inherit zmap; };

        scan-icmp-time = pkgs.writeShellApplication {
          name = "scan-icmp-time";
          runtimeInputs = [ zmapPackage pkgs.dig ];
          text = builtins.readFile ./icmp_clocksync/scripts/scan-icmp-time.sh;
        };

        scan-http = pkgs.writeShellApplication {
          name = "scan-http";
          runtimeInputs = [ zmapPackage pkgs.dig pkgs.gnused ];
          text = builtins.readFile ./icmp_clocksync/scripts/scan-http.sh;
        };

        treefmtconfig = inputs.treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            rustfmt.enable = true;
            nixfmt.enable = true;
            clang-format.enable = true;
            shfmt.enable = true;
            ruff-format.enable = true;
            ruff-check.enable = true;
          };
        };
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages = [
              rustToolchain
            ] ++ (with pkgs; [
              just
              openssl.dev
              zmapPackage
              gcc
              clang-tools
              bear
              libpcap
              zlib.dev
              gmp
              gengetopt
              flex
              byacc
              json_c
              libunistring
              pkg-config
              quarto
              dig
              inetutils
              uv
              ruff
            ]);
            shellHook = ''
              export CC=${pkgs.gcc}/bin/gcc
              export ZMAP_SRC="${zmap}/src"
              export C_INCLUDE_PATH="${zmap}/src:${pkgs.zlib.dev}/include:${pkgs.libpcap}/include:${pkgs.json_c}/include:${pkgs.gmp.dev}/include"
              bear -- make -C ./icmp_clocksync clean && bear -- make -C ./icmp_clocksync
            '';
          };
        };
        apps = {
          scan-icmp-time = {
            type = "app";
            program = "${scan-icmp-time}/bin/scan-icmp-time";
          };
          scan-http = {
            type = "app";
            program = "${scan-http}/bin/scan-http";
          };
        };
        packages = {
          inherit scan-icmp-time scan-http;
        };
        formatter = treefmtconfig.config.build.wrapper;
        checks = {
          formatting = treefmtconfig.config.build.check self;
        };
      }
    );
}
