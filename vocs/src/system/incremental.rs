use mask::{ChunkMask, LayerMask};
use view::SpillChunk;
use world::world::World;
use parking_lot::Mutex;
use view::Directional;

// TODO: Incremental dispatcher.

type IncomingChunkMask = Directional<Option<LayerMask>>;

struct WorldQueue {
	queue: World<Box<Mutex<IncomingChunkMask>>>
}

struct IncrementalDispatcher<I, S> where I: Incremental<SystemData=S> {
	queue: WorldQueue,
	data: (),
	system: I
}

impl<I, S> IncrementalDispatcher<I, S> where I: Incremental<SystemData=S> {

}

trait Incremental {
	type SystemData;

	fn run(&self, data: Self::SystemData, current: &mut ChunkMask, future: &mut SpillChunk<bool>);
}