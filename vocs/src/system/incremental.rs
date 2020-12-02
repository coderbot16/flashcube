use mask::{BitCube, LayerMask};
use view::SpillChunk;
use world::world::World;
use parking_lot::Mutex;
use view::Directional;

// TODO: Incremental dispatcher.

type IncomingBitCube = Directional<Option<LayerMask>>;

struct WorldQueue {
	queue: World<Box<Mutex<IncomingBitCube>>>
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

	fn run(&self, data: Self::SystemData, current: &mut BitCube, future: &mut SpillChunk<bool>);
}