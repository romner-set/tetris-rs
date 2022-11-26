# tetris-rs
A (hopefully) cross-platform Tetris clone for the CLI.  
[preview](preview.gif)

## Features
- Standard Tetris stuff, e.g. 7 pieces, piece holding, ghost pieces, etc.
- Written purely in Rust
- Command line arguments to customize controls, speed, etc. (scale doesn't really work at the moment)
- Doesn't switch to an alternate window, runs directly where you type the command
- Cleans up after itself

## Build from source
1. Install [Rust](https://www.rust-lang.org/tools/install)
2. `cd` into a folder of your choice and run `git clone https://github.com/romner-set/tetris-rs.git`
3. `cd` into the resulting directory and run `cargo build --release`
4. The executable should now be in `target/release/`
