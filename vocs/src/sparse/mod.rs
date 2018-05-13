use component::*;
use fxhash::FxHashMap;
use std::collections::hash_map::Entry;
use position::LayerPosition;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct SparseStorage<C> where C: Component + Eq {
	pages: FxHashMap<usize, C::Layer>,
	default: C
}

impl<C> SparseStorage<C> where C: Component + Eq {
	pub fn with_default(default: C) -> Self {
		SparseStorage {
			pages: FxHashMap::default(),
			default
		}
	}

	pub fn get(&self, index: usize) -> C {
		let (page, inner) = (
			index / 256,
			LayerPosition::from_zx((index % 256) as u8)
		);

		self.pages.get(&page).map(|page| page.get(inner)).unwrap_or_else(|| self.default.clone())
	}

	pub fn set(&mut self, index: usize, v: C) {
		let (page, inner) = (
			index / 256,
			LayerPosition::from_zx((index % 256) as u8)
		);

		if v == self.default {
			match self.pages.entry(page) {
				Entry::Occupied(mut occupied) => {
					occupied.get_mut().set(inner, v);

					if occupied.get().is_filled(self.default.clone()) {
						occupied.remove();
					}
				},
				_ => ()
			}
		} else {
			let default = self.default.clone();

			self.pages.entry(page).or_insert_with(|| {
				let mut page = C::Layer::default();

				page.fill(default);

				page
			}).set(inner, v);
		}
	}

	pub fn clear(&mut self) {
		self.pages.clear();
	}

	pub fn default_value(&self) -> C {
		self.default.clone()
	}
}