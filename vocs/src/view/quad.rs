use indexed::Target;
use position::QuadPosition;
use super::{ColumnAssociation, ColumnBlocks, ColumnMut, ColumnPalettes};

#[derive(Debug)]
pub struct QuadMut<'c, B>(pub [ColumnMut<'c, B>; 4]) where B: 'c + Target;

impl<'c, B> QuadMut<'c, B> where B: 'c + Target {
	pub fn get(&self, at: QuadPosition) -> Option<&B> {
		let q = at.q() as usize;

		self.0[q].get(at.column())
	}

	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// As a rule of thumb, this should be faster if you know you are setting less than 16 blocks.
	pub fn set_immediate(&mut self, position: QuadPosition, target: &B) {
		let q = position.q() as usize;

		self.0[q].set_immediate(position.column(), target)
	}

	/// Makes sure that a future lookup for the target will succeed, unless the entry has changed since this call.
	pub fn ensure_available(&mut self, target: B) {
		self.0[0].ensure_available(target.clone());
		self.0[1].ensure_available(target.clone());
		self.0[2].ensure_available(target.clone());
		self.0[3].ensure_available(target.clone());
		
	}

	pub fn freeze_palette(&mut self) -> (QuadBlocks, QuadPalettes<B>) {
		let columns = slice_to_tuple_mut_4(&mut self.0);

		let frozen = (
			columns. 0.freeze_palette(), columns. 1.freeze_palette(), columns. 2.freeze_palette(), columns. 3.freeze_palette()
		);

		(
			QuadBlocks ([
				(frozen. 0).0, (frozen. 1).0, (frozen. 2).0, (frozen. 3).0
			]),
			QuadPalettes([
				(frozen. 0).1, (frozen. 1).1, (frozen. 2).1, (frozen. 3).1
			])
		)
	}
}

#[derive(Debug)]
pub struct QuadBlocks<'a>([ColumnBlocks<'a>; 4]);
impl<'a> QuadBlocks<'a> {
	pub fn get<'p, B>(&self, at: QuadPosition, palettes: &QuadPalettes<'p, B>) -> Option<&'p B> where B: Target {
		let q = at.q() as usize;

		self.0[q].get(at.column(), &palettes.0[q])
	}

	pub fn set(&mut self, at: QuadPosition, association: &QuadAssociation) {
		let q = at.q() as usize;

		self.0[q].set(at.column(), &association.0[q])
	}
}

#[derive(Debug)]
pub struct QuadPalettes<'a, B>([ColumnPalettes<'a, B>; 4]) where B: 'a + Target;
impl<'a, B> QuadPalettes<'a, B> where B: 'a + Target {
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Result<QuadAssociation, ()> {
		Ok(QuadAssociation ([
			self.0 [0].reverse_lookup(target).map_err(|_| ())?,
			self.0 [1].reverse_lookup(target).map_err(|_| ())?,
			self.0 [2].reverse_lookup(target).map_err(|_| ())?,
			self.0 [3].reverse_lookup(target).map_err(|_| ())?
		]))
	}
}

#[derive(Debug)]
pub struct QuadAssociation([ColumnAssociation; 4]);

pub fn slice_to_tuple_mut_4<T>(slice: &mut [T; 4]) -> (&mut T, &mut T, &mut T, &mut T)
{
	let (s0, slice) = slice.split_at_mut(1);
	let (s1, slice) = slice.split_at_mut(1);
	let (s2, s3   ) = slice.split_at_mut(1);

	(&mut s0 [0], &mut s1 [0], &mut s2 [0], &mut s3 [0])
}