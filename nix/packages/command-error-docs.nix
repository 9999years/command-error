{command-error}: let
  inherit
    (command-error)
    craneLib
    commonArgs
    ;
in
  craneLib.cargoDoc (commonArgs
    // {
      # The default `cargoDocExtraArgs` is `--no-deps`.
      cargoDocExtraArgs = "--all-features";
      RUSTDOCFLAGS = "-D warnings";
    })
