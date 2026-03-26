{ makeDesktopItem
, rustPlatform
, lib
, pkg-config
, wayland-scanner
, scdoc
, glib
, gdk-pixbuf
, pango
, cairo
, gtk4
, wayland
, wayland-protocols
, ffmpeg
, x264
, libpulseaudio
, pipewire
, mesa
, wrapGAppsHook4
}:

rustPlatform.buildRustPackage rec {
  pname = "recway";
  version = "0.4.0";

  src = ./.;

  useFetchCargoVendor = true;
  cargoHash = "sha256-OjALvs+JdObN4SPGffVb4e8OFvE1HxPP+stA22XFPKs=";

  nativeBuildInputs = [
    pkg-config
    wayland-scanner
    scdoc
    wrapGAppsHook4
  ];

  buildInputs = [
    wayland
    wayland-protocols
    ffmpeg
    x264
    libpulseaudio
    pipewire
    mesa
    glib
    gdk-pixbuf
    pango
    cairo
    gtk4
  ];

  desktopEntry = [
    (makeDesktopItem {
      name = "RecWay";
      comment = "A frontend for wf-recorder screen recorder";
      exec = "recway";
      icon = "camera-video-symbolic";
      desktopName = "RecWay";
      terminal = false;
      type = "Application";
      categories = [ "AudioVideo" "Video" "Recorder" "GTK" ];
      keywords = [ "screen" "recorder" "wayland" "capture" ];
      startupNotify = true;
    })
  ];

  postInstall = ''
    mkdir -p $out/share/applications
    for entry in ${toString desktopEntry}; do
      cp $entry/share/applications/*.desktop $out/share/applications/
    done
  '';

  meta = with lib; {
    description = "A frontend for wf-recorder, the Wayland screen recorder";
    homepage = "https://github.com/nabiko02/recway";
    license = licenses.mit;
    maintainers = [ ];
  };
}
