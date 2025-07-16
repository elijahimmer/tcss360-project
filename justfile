run:
  cargo fmt
  cargo run  -F debug,dev,bevy/wayland

x11:
  cargo fmt
  cargo run --profile x11 -F debug,dev

release:
  cargo build --release

wasm:
  trunk serve --cargo-profile wasm --no-default-features --features debug

wasm-release:
  -rm game.zip
  trunk build --cargo-profile wasm-release --no-default-features
  zip game.zip dist -r

wasm-release-run: wasm-release
  trunk serve --cargo-profile wasm-release --no-default-features

clean:
  -rm game.zip result
  trunk clean
  cargo clean
