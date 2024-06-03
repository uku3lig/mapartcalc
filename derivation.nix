{
  lib,
  rustPlatform,
  self,
}:
rustPlatform.buildRustPackage {
  pname = "api-rs";
  version = self.shortRev or self.dirtyShortRev or "dirty";

  cargoLock.lockFile = ./Cargo.lock;

  src = self;

  meta = with lib; {
    mainProgram = "mapartcalc";
    license = licenses.mit;
    platforms = platforms.unix;
  };
}
