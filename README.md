# Shoko Rocket Rust
A clone of Chu Chu Rocket, targeting the Adafruit Pygamer

## Structure
There are three main components
* common. This has structures shared by all projects
* simulation. This is platform independent code that simulates the state of the game.
* platform. This is glue code that is responsible for calling the simulation code, then rendering the results.
* bin. This is the entrypoint

bin -> simulation -> common
                  -> world_macros
    -> platform   -> common
    -> common
    -> render -> simulation
              -> platform
              -> common

## Adafruit Pygamer
To build execute `cargo build`. The target architecture will automatically be set by `.cargo/config`.

## Building and installing
To install you need to have the tool (https://crates.io/crates/hf2-cli)[hf2-cli] on the path

`cargo install hf2-cli`

After this you can execute `cargo run --release` and the built image will be uploaded to the attached pygamer.

## Tests
To run unit tests under the host architecture run `cargo test --target=x86_64-pc-windows-msvc --lib`,
replacing your target as required.

