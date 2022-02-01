# Shoko Rocket Rust
A clone of Chu Chu Rocket, targetting the Adafruit Pygamer

## Adafruit Pygamer
To build execute `cargo build`. The target architecture will automatically be set by `.cargo/config`.

## Building and installing
To install you need to have the tool (https://crates.io/crates/hf2-cli)[hf2-cli] on the path

`cargo install hf2-cli`

After this you can execute `cargo run --release` and the built image will be uploaded to the attached pygamer.

## Tests
To run unit tests under the host architecture run `cargo test --target=x86_64-pc-windows-msvc --lib`,
replacing your target as required.

