
wasm:
  trunk serve --no-default-features

wasm-release:
  rm game.zip
  trunk build --cargo-profile wasm-release --no-default-features
  zip game.zip dist -r

clean:
  trunk clean
  cargo clean
