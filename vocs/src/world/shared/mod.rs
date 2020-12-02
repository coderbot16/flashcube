pub mod sector;
pub mod world;

pub use self::sector::SharedSector;
pub use self::world::SharedWorld;

use spin::RwLockWriteGuard;
use std::ops::{Deref, DerefMut};

pub trait Packed {
	type Unpacked;

	fn unpack(self) -> Self::Unpacked;
	fn pack(unpacked: Self::Unpacked) -> Self;
}

pub struct NoPack<T>(pub T);

impl<T> Packed for NoPack<T> {
	type Unpacked = T;

	fn unpack(self) -> T {
		self.0
	}

	fn pack(unpacked: T) -> Self {
		NoPack(unpacked)
	}
}

impl<T> Default for NoPack<T> where T: Default {
	fn default() -> Self {
		NoPack(T::default())
	}
}

pub struct Guard<'s, T> where T: 's + Packed {
	slot: RwLockWriteGuard<'s, Option<T>>,
	value: Option<T::Unpacked>
}

impl<'s, T> Deref for Guard<'s, T> where T: 's + Packed {
	type Target = T::Unpacked;

	fn deref(&self) -> &T::Unpacked {
		self.value.as_ref().unwrap()
	}
}

impl<'s, T> DerefMut for Guard<'s, T> where T: 's + Packed {
	fn deref_mut(&mut self) -> &mut T::Unpacked {
		self.value.as_mut().unwrap()
	}
}

impl<'s, T> Drop for Guard<'s, T> where  T: 's + Packed {
	fn drop(&mut self) {
		*self.slot = Some(T::pack(self.value.take().unwrap()));
	}
}