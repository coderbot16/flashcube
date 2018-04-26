use component::*;
use fxhash::FxHashMap;
use std::collections::hash_map::Entry;
use position::LayerPosition;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SparseStorage<C> where C: Component + Eq {
	pages: FxHashMap<usize, C::Layer>
}

impl<C> SparseStorage<C> where C: Component + Eq {
	pub fn get(&self, index: usize) -> C {
		let (page, inner) = (
			index / 256,
			LayerPosition::from_zx((index % 256) as u8)
		);

		self.pages.get(&(index / 256)).map(|page| page.get(inner)).unwrap_or_else(C::default)
	}

	pub fn set(&mut self, index: usize, v: C) {
		let (page, inner) = (
			index / 256,
			LayerPosition::from_zx((index % 256) as u8)
		);

		if v == C::default() {
			match self.pages.entry(page) {
				Entry::Occupied(mut occupied) => {
					occupied.get_mut().set(inner, C::default());

					if occupied.get().is_empty() {
						occupied.remove();
					}
				},
				_ => ()
			}
		} else {
			self.pages.entry(page).or_insert_with(C::Layer::default).set(inner, v);
		}
	}

	pub fn clear(&mut self) {
		self.pages.clear();
	}
}