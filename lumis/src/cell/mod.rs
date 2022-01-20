mod kernel;
mod u4x16;
mod storage;

//! Word-level parallelism inspired by / based on https://0fps.net/2018/02/21/voxel-lighting, by Mikola Lysenko
//!
//! Notable differences include the fact that we store multiple values into a single "word" spatially, where each
//! word contains nearby light values in a cell, but the artice stores different channels of different lightmaps into
//! a single word.
//!
//! We represent the lightmap as a grid of cells, where each cell is a 2x2x2 collection of nibbles. On each iteration,
//! we dequeue two non-adjacent cells, and fetch both from the lightmaps / opacity maps into a single u4x16 value. We
//! end up fetching the following information:
//! 
//! - 1 u4x16, cell lightmap
//! - 1 u4x16, cell emission
//! - 1 u4x16, cell opacities
//! - 6 u4x16, neighboring cells
//!
//! All in all, we read 72 bytes from memory and write 8 bytes to memory on each iteration. With a more naive approach
//! operating on nibbles only, we'd read way less data, but will likely spend around the same time as it would take to
//! fetch two nibbles from memory, since to fetch a nibble we have to read an entire machine word (or even a cache line)
//! just to grab the 4 bit value. This gives us a cool (theoretical) 8x speedup in the fetch phase.
//!
//! Since the smallest unit is a 2x2x2 cell of 8 nibbles, instead of requiring 8192 bits / 1024 bytes to store the queue
//! data for each chunk, we only require 1024 bits / 128 bytes (64 bytes per side). This means that not only does each
//! side of the queue fit in a single 64-byte cache line, but we can also conveniently use a 64-bit mask value where each bit
//! represents whether a given byte has any value enqueued.
