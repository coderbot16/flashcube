# nbt-turbo

`nbt-turbo` is a tiny and fast NBT encoder for Rust. It is designed with the following goals in mind:

 * Minimal compile time
 * Fast encoding
 * Simple and ergonomic API

`nbt-turbo` is different from crates such as `serde` in that it does not provide the user with compile time serialization
code generation, however, such features do come at a cost: inclusion of `serde` has been found to balloon the compile
times of my personal projects, making it a pain to work with on fast-moving projects.