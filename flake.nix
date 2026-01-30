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

        zmapPackage = pkgs.stdenv.mkDerivation {
          pname = "zmap";
          version = "4.3.0";
          src = zmap;

          nativeBuildInputs = with pkgs; [
            cmake
            pkg-config
            flex
            byacc
            gengetopt
          ];

          buildInputs = with pkgs; [
            libpcap
            gmp
            json_c
            libunistring
            judy
          ];

          # Copy custom probe module into zmap source tree before building
          preConfigure = ''
            cp ${./zmap/module_icmp_timestamp.c} src/probe_modules/module_icmp_timestamp.c

            # Add the module to CMakeLists.txt
            sed -i '/probe_modules\/module_icmp_echo_time.c/a\    probe_modules/module_icmp_timestamp.c' src/CMakeLists.txt

            # Register the module in probe_modules.c
            sed -i '/extern probe_module_t module_icmp_echo_time;/a\extern probe_module_t module_icmp_timestamp;' src/probe_modules/probe_modules.c
            sed -i '/\&module_icmp_echo_time,/a\    \&module_icmp_timestamp,' src/probe_modules/probe_modules.c
          '';

          cmakeFlags = [
            "-DENABLE_DEVELOPMENT=OFF"
            "-DRESPECT_INSTALL_PREFIX_CONFIG=ON"
          ];

          meta = {
            description = "Fast Internet-wide scanner";
            homepage = "https://zmap.io";
            license = pkgs.lib.licenses.asl20;
          };
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
            ];

            shellHook = ''
              export CC=${pkgs.gcc}/bin/gcc
              export ZMAP_SRC="${zmap}/src"
              export C_INCLUDE_PATH="${zmap}/src:${pkgs.zlib.dev}/include:${pkgs.libpcap}/include:${pkgs.json_c}/include:${pkgs.gmp.dev}/include"
              bear -- make clean && bear -- make
            '';
          };
        };
        formatter = treefmtconfig.config.build.wrapper;
        checks = {
          formatting = treefmtconfig.config.build.check self;
        };
      }
    );
}
