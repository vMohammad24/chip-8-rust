{pkgs, ...}: {
  languages.rust.enable = true;

  packages = with pkgs; [
    pkg-config
    wayland
    libxkbcommon
    libXcursor
    libXi
    alsa-lib
  ];
}
