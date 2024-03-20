{
  stdenv,
  command-error-docs,
}: let
  inherit (command-error-docs) version;
in
  stdenv.mkDerivation {
    pname = "command-error-docs-tarball";
    inherit version;

    src = command-error-docs;

    dontConfigure = true;
    dontBuild = true;

    installPhase = ''
      dir=command-error-docs-${version}
      mv share/doc \
        "$dir"

      mkdir $out
      tar --create \
        --file $out/command-error-docs-${version}.tar.gz \
        --auto-compress \
        --verbose \
        "$dir"
    '';
  }
