{
  description = "CSDS426 Clock Project";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
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

        zmapPackage = pkgs.callPackage ./zmap-scanning/build.nix { inherit zmap; };

        scan-icmp-time = pkgs.writeShellApplication {
          name = "scan-icmp-time";
          runtimeInputs = [ zmapPackage pkgs.dig ];
          text = builtins.readFile ./zmap-scanning/scripts/scan-icmp-time.sh;
        };

        scan-http = pkgs.writeShellApplication {
          name = "scan-http";
          runtimeInputs = [ zmapPackage pkgs.dig pkgs.gnused ];
          text = builtins.readFile ./zmap-scanning/scripts/scan-http.sh;
        };

        treefmtconfig = inputs.treefmt-nix.lib.evalModule pkgs {
          projectRootFile = "flake.nix";
          programs = {
            rustfmt.enable = true;
            nixfmt.enable = true;
            clang-format.enable = true;
            shfmt.enable = true;
          };
        };
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages = with pkgs; [
              openssl.dev
              zmapPackage
              cargo
              rust-analyzer
              rustc
              rustfmt
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
            ];
            shellHook = ''
              export CC=${pkgs.gcc}/bin/gcc
              export ZMAP_SRC="${zmap}/src"
              export C_INCLUDE_PATH="${zmap}/src:${pkgs.zlib.dev}/include:${pkgs.libpcap}/include:${pkgs.json_c}/include:${pkgs.gmp.dev}/include"
              bear -- make -C ./zmap-scanning clean && bear -- make -C ./zmap-scanning
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
