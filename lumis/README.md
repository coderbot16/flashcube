# Lumis

An incredibly fast library for computing Minecraft lighting using `vocs`.

Lumis achieves both accuracy and high performance by using an efficient BFS-based algorithm that operates directly on chunks. Lumis can compute the lighting for an entire region file in around 125 milliseconds, where comparable algorithms like the one in MCEdit would take over a minute. Lumis uses multiple threads to compute lighting data, allowing the speed to quickly scale with the number of available CPU cores. Even on my i7-4790 system with only 4 cores, I'm able to obtain a 3.3x speedup in many cases compared to if there were no multithreading.

## Why?

The [Phosphor](https://www.curseforge.com/minecraft/mc-mods/phosphor) mod for Minecraft has already revealed that the Minecraft lighting algorithm is a significant performance bottleneck. By designing a fast voxel lighting engine from the ground up, I hope to completely negate the performance issues of lighting within my own engine.

Furthermore, the algorithms used in Minecraft lighting are common in similar other game mechanics, such as fluid flow and redstone. Thus, optimizations to lighting code will almost certainly be reusable elsewhere.


## History

The lighting code originated in the i73 project around October 2017. Only about 250 lines long, it still had the same core design that Lumis now has. The biggest changes since then have been a significant cleanup of all of the related code, followed by significant optimization work.


## Future Work

Lumis works very well, however the code is is need of cleanup. Furthermore, Lumis does not currently support incrementally updating the lighting data of individual chunks, only computing new data from scratch. It's entirely possible to extend the code to support this, however.


## Credits

- Word-level parallelism from https://0fps.net/2018/02/21/voxel-lighting/, by Mikola Lysenko
