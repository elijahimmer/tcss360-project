run:
  cargo run -F debug,dev

wayland:
  cargo run --profile wayland -F debug,dev,bevy/wayland

release:
  cargo build --release

wasm:
  trunk serve --no-default-features --features debug

wasm-release:
  rm game.zip
  trunk build --cargo-profile wasm-release --no-default-features
  zip game.zip dist -r

clean:
  -rm game.zip result
  trunk clean
  cargo clean
