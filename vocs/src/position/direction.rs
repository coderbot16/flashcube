pub trait Offset<D>: Sized {
	fn offset(self, dir: D) -> Option<Self>;
	fn offset_wrapping(self, dir: D) -> Self;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Dir {
	Up,
	Down,
	PlusX,
	MinusX,
	PlusZ,
	MinusZ
}

impl Dir {
	pub fn opposite(self) -> Dir {
		match self {
			Dir::Up     => Dir::Down,
			Dir::Down   => Dir::Up,
			Dir::PlusX  => Dir::MinusX,
			Dir::MinusX => Dir::PlusX,
			Dir::PlusZ  => Dir::MinusZ,
			Dir::MinusZ => Dir::PlusZ
		}
	}

	pub fn horizontal(self) -> bool {
		!self.vertical()
	}

	pub fn vertical(self) -> bool {
		self == Dir::Up || self == Dir::Down
	}

	pub fn axis(self) -> Axis {
		match self {
			Dir::Up     => Axis::Y,
			Dir::Down   => Axis::Y,
			Dir::PlusX  => Axis::X,
			Dir::MinusX => Axis::X,
			Dir::PlusZ  => Axis::Z,
			Dir::MinusZ => Axis::Z
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Axis {
	Y,
	X,
	Z
}

impl Axis {
	pub fn horizontal(self) -> bool {
		!self.vertical()
	}

	pub fn vertical(self) -> bool {
		self == Axis::Y
	}

	pub fn plus(self) -> Dir {
		match self {
			Axis::Y => Dir::Up,
			Axis::X => Dir::PlusX,
			Axis::Z => Dir::PlusZ
		}
	}

	pub fn minus(self) -> Dir {
		match self {
			Axis::Y => Dir::Down,
			Axis::X => Dir::MinusX,
			Axis::Z => Dir::MinusZ
		}
	}
}

// Direction types

pub trait StaticDirection: Into<Dir> {
	type Opposite: StaticDirection;
	type Axis: StaticAxis;
}

pub trait StaticAxis {
	type Plus: StaticDirection;
	type Minus: StaticDirection;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Up;
impl StaticDirection for Up {
	type Opposite = Down;
	type Axis = Y;
}

impl From<Up> for Dir {
	fn from(_: Up) -> Dir {
		Dir::Up
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Down;
impl StaticDirection for Down {
	type Opposite = Up;
	type Axis = Y;
}

impl From<Down> for Dir {
	fn from(_: Down) -> Dir {
		Dir::Down
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlusX;
impl StaticDirection for PlusX {
	type Opposite = MinusX;
	type Axis = X;
}

impl From<PlusX> for Dir {
	fn from(_: PlusX) -> Dir {
		Dir::PlusX
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MinusX;
impl StaticDirection for MinusX {
	type Opposite = PlusX;
	type Axis = X;
}

impl From<MinusX> for Dir {
	fn from(_: MinusX) -> Dir {
		Dir::MinusX
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PlusZ;
impl StaticDirection for PlusZ {
	type Opposite = MinusZ;
	type Axis = Z;
}

impl From<PlusZ> for Dir {
	fn from(_: PlusZ) -> Dir {
		Dir::PlusZ
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct MinusZ;
impl StaticDirection for MinusZ {
	type Opposite = PlusZ;
	type Axis = Z;
}

impl From<MinusZ> for Dir {
	fn from(_: MinusZ) -> Dir {
		Dir::MinusZ
	}
}

// Axis types

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct X;
impl StaticAxis for X {
	type Plus = PlusX;
	type Minus = MinusX;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Y;
impl StaticAxis for Y {
	type Plus = Up;
	type Minus = Down;
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Z;
impl StaticAxis for Z {
	type Plus = PlusZ;
	type Minus = MinusZ;
}