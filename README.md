# flashcube

A fast parallel voxel engine for Minecraft-like worlds written in Rust

## Compiling and running

1. [Install Rust and Cargo, 1.48 or later is required](https://www.rust-lang.org/tools/install)
2. Execute `cargo build --release` in the terminal of your choice
3. (Optional) Execute `cargo run --release --bin <name>` to run a given program
	* `main` generates a single region file of a Minecraft Beta 1.7.3 world
	* `mapper` creates images of Minecraft Beta 1.7.3 worlds


## Project structure

This project uses a [monorepo](https://en.wikipedia.org/wiki/Monorepo) style of organization where each individual crate/module is stored in the same main source tree. A [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) is used to join the individual crates together.

* [`vocs`](vocs/README.md): A voxel component system that provides primitives for constructing voxel engines
* [`lumis`](lumis/README.md): An extremely fast parallel voxel flood-fill lighting engine that takes advantage of multiple CPU cores/threads
	* To my knowledge, this is the fastest CPU flood-fill lighting engine in existence, and is likely faster than [Starlight](https://github.com/Spottedleaf/Starlight) while introducing absolutely no lighting errors
* [`nbt-turbo`](nbt-turbo/README.md): A tiny library for writing NBT files that aims for minimal compile times and zero code bloat
* [`mclevel`](mclevel/README.md): A set of crates for writing Minecraft level files in various formats (Anvil, ClassicWorld, etc)
* [`i73`](i73/README.md): A world generator that generates terrain that is almost entirely identical to Beta 1.7.3, serves as a test bench for all of the previous projects


## License

Most crates are licensed under the GNU GPLv3 license, but a few components that are useful elsewhere are offered under the MIT license as well.
