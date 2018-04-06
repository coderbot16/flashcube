use mask::{ChunkMask, LayerMask};
use mask::spill::SpillChunkMask;
use world::world::World;
use parking_lot::Mutex;

// TODO: Incremental dispatcher.

#[derive(Clone)]
struct IncomingChunkMask {
	pub up:      Option<LayerMask>,
	pub down:    Option<LayerMask>,
	pub plus_x:  Option<LayerMask>,
	pub minus_x: Option<LayerMask>,
	pub plus_z:  Option<LayerMask>,
	pub minus_z: Option<LayerMask>
}

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

	fn run(&self, data: Self::SystemData, current: &mut ChunkMask, future: &mut SpillChunkMask);
}