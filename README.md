# TCSS 360 project
(name WIP)

## Compilation
### Linux

If you have `nix` (https://nixos.org/), you can
simplify do `nix build` and it should build properly.

Otherwise,
This project requires the libraries

- `alsa-lib`
- `vulkan`
- `vulkan-loader`
- `libX11`
- `libXcursor`
- `libXi`
- `libXrandr`
- `libxkbcommon`
- `udev`

And the following tools:
- `rustc`
- `cargo`
- `pkg-config`
- `cmake`

Once all of these are available (you should have many already installed), run:
```sh
cargo run
```

This will download all of the rust dependencies

### Mac
WIP

### Windows
WIP
