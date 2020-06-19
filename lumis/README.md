# Lumis

An incredibly fast library for computing Minecraft lighting using `vocs`.

Lumis achieves both accuracy and high performance by using an efficient BFS-based algorithm that operates directly on chunks. Lumis can compute the lighting for an entire region file in around 1 second, where comparable algorithms like the one in MCEdit would take over a minute. Lumis has been designed specifically to support incredibly scalable threading, which would bring another order-of-magnitude speedup on most systems.

## Why?

The [Phosphor](https://www.curseforge.com/minecraft/mc-mods/phosphor) mod for Minecraft has already revealed that the Minecraft lighting algorithm is a significant performance bottleneck. By designing a fast voxel lighting engine from the ground up, I hope to completely negate the performance issues of lighting within my own engine.

Furthermore, the algorithms used in Minecraft lighting are common in similar other game mechanics, such as fluid flow and redstone. Thus, optimizations to lighting code will almost certainly be reusable elsewhere.


## History

The lighting code originated in the i73 project around October 2017. Only about 250 lines long, it still had the same core design that Lumis now has. The biggest changes since then have been a significant cleanup of all of the related code, followed by significant optimization work.


## Future Work

Lumis is still primarily used in a single-threaded fashion, even if it theoretically supports multithreading. In the process of cleaning up the code, I wish to make multithreaded usage simple and easy.
