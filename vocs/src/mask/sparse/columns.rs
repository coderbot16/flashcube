use crate::position::{GlobalSectorPosition, GlobalColumnPosition};
use crate::mask::{Mask, LayerMask, Scan, ScanClear};
use std::collections::HashMap;
use std::collections::hash_map::{Entry, Iter, IterMut};
use std::ops::Index;
use crate::component::*;

const FALSE_REF: &bool = &false;

pub struct ColumnsMask(HashMap<GlobalSectorPosition, LayerMask>);

impl ColumnsMask {
	pub fn new() -> Self {
		ColumnsMask(HashMap::new())
	}

	pub fn sectors(&self) -> Iter<GlobalSectorPosition, LayerMask> {
		self.0.iter()
	}

	// TODO: If the user clears the masks returned by iter_mut, the ColumnsMask will never remove them.
	pub fn sectors_mut(&mut self) -> IterMut<GlobalSectorPosition, LayerMask> {
		self.0.iter_mut()
	}

	pub fn sector(&self, coordinates: GlobalSectorPosition) -> Option<&LayerMask> {
		self.0.get(&coordinates)
	}

	pub fn clear_sector(&mut self, coordinates: GlobalSectorPosition) {
		self.0.remove(&coordinates);
	}

	pub fn fill_sector(&mut self, coordinates: GlobalSectorPosition) {
		let mut mask = LayerMask::default();
		mask.fill(true);

		self.0.insert(coordinates, mask);
	}

	pub fn clear(&mut self) {
		self.0.clear();
	}
}

impl Mask<GlobalColumnPosition> for ColumnsMask {
	fn set_false(&mut self, column: GlobalColumnPosition) {
		let (sector, position) = (column.global_sector(), column.local_layer());

		if let Entry::Occupied(mut entry) = self.0.entry(sector) {
			let remove = {
				let mask = entry.get_mut();

				mask.set_false(position);
				mask.count_ones() == 0
			};

			if remove {
				entry.remove();
			}
		}
	}

	fn set_true(&mut self, column: GlobalColumnPosition) {
		let (sector, position) = (column.global_sector(), column.local_layer());

		self.0.entry(sector).or_insert(LayerMask::default()).set_true(position);
	}

	fn set_or(&mut self, column: GlobalColumnPosition, value: bool) {
		if value {
			self.set_true(column);
		}
	}

	fn scan(&self) -> Scan<Self, GlobalColumnPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return sectors instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn scan_clear(&mut self) -> ScanClear<Self, GlobalColumnPosition> {
		// TODO: Scanning sparse maps has a non deterministic order. Return sectors instead?
		unimplemented!("No clear / logical way to scan a sparse map yet...")
	}

	fn count_ones(&self) -> u32 {
		self.0.values().fold(0, |state, value| state + value.count_ones())
	}

	fn count_zeros(&self) -> u32 {
		self.0.values().fold(0, |state, value| state + value.count_zeros())
	}
}

impl Index<GlobalColumnPosition> for ColumnsMask {
	type Output = bool;

	fn index(&self, column: GlobalColumnPosition) -> &bool {
		let (sector, inner) = (column.global_sector(), column.local_layer());

		self.0.get(&sector).map(|sector| &sector[inner]).unwrap_or(FALSE_REF)
	}
}