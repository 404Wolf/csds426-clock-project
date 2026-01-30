{
  stdenv,
  zmap,
  cmake,
  pkg-config,
  flex,
  byacc,
  gengetopt,
  libpcap,
  gmp,
  json_c,
  libunistring,
  judy,
  lib,
}:
stdenv.mkDerivation {
  pname = "zmap";
  version = "4.3.0";
  src = zmap;

  nativeBuildInputs = [
    cmake
    pkg-config
    flex
    byacc
    gengetopt
  ];

  buildInputs = [
    libpcap
    gmp
    json_c
    libunistring
    judy
  ];

  # Copy custom probe module into zmap source tree before building
  preConfigure = ''
    cp ${./module/module_icmp_timestamp.c} src/probe_modules/module_icmp_timestamp.c

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
    license = lib.licenses.asl20;
  };
}
