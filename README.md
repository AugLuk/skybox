# Skybox
A configurable skybox generator with procedurally generated clouds.

Can also be used as a CPU / WASI runtime benchmarking tool.

## Previews

![skybox](previews/skybox.png)
*The sun is perfectly round when the box is "folded". The image is brightened in this preview*

![render](previews/render.png)
*A render using the skybox above as the only light source*

## Installation and Running
Requires a bash (or equivalent) command line, git and Rust toolchain.

### Preparation - Getting the Source Code (for all builds)

```bash
git clone https://github.com/AugLuk/skybox.git
cd skybox
```

### Native Build

#### Optimized
```bash
RUSTFLAGS='-C target-cpu=native' cargo build --profile release-lto
./target/release-lto/skybox
```

#### Debug
```bash
cargo build
./target/debug/skybox
```

### WASI Build and Running in Wasmtime
Additionally requires the cargo-wasi subcommand and wasmtime.

#### Optimized
```bash
cargo wasi build --profile release-lto --features no-multithreading
wasmtime run --dir=. target/wasm32-wasi/release-lto/skybox.wasm
```

## Additional Directions

Edit the *config.ini* file to change the simulation parameters.

The output images are saved in the *output/* directory.

It is necessary to move the images to another location or rename them to prevent the application from overwriting them the next time it is run.
