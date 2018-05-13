use position::Dir;

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct SplitDirectional<T> {
	pub up:      T,
	pub down:    T,
	pub plus_x:  T,
	pub minus_x: T,
	pub plus_z:  T,
	pub minus_z: T
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Directional<T>([T; 6]);

impl<T> Directional<T> {
	pub fn combine(split: SplitDirectional<T>) -> Self {
		Directional ([
			split.up,
			split.down,
			split.plus_x,
			split.minus_x,
			split.plus_z,
			split.minus_z
		])
	}

	pub fn new(up: T, down: T, plus_x: T, minus_x: T, plus_z: T, minus_z: T) -> Self {
		Directional([up, down, plus_x, minus_x, plus_z, minus_z])
	}
	
	pub fn get(&self, direction: Dir) -> &T {
		&self.0[direction as usize]
	}
	
	pub fn get_mut(&mut self, direction: Dir) -> &mut T {
		&mut self.0[direction as usize]
	}
	
	pub fn split(self) -> SplitDirectional<T> {
		// TODO: Update to 1.26 for slice patterns
		/*match self.0 {
			[up, down, plus_x, minus_x, plus_z, minus_z] => unimplemented!()
		}*/

		unimplemented!()
	}
}