use indexed::{Target, ChunkIndexed};
use indexed::Palette;
use position::ColumnPosition;
use packed::ChunkPacked;

#[derive(Debug)]
pub struct ColumnMut<'c, B>(pub &'c mut [ChunkIndexed<B>; 16]) where B: 'c + Target;

impl<'c, B> ColumnMut<'c, B> where B: 'c + Target {
	pub fn get(&self, at: ColumnPosition) -> Option<&B> {
		let chunk_y = at.chunk_y() as usize;

		self.0[chunk_y].get(at.chunk())
	}

	/// Preforms the ensure_available, reverse_lookup, and set calls all in one.
	/// As a rule of thumb, this should be faster if you know you are setting less than 16 blocks.
	pub fn set_immediate(&mut self, position: ColumnPosition, target: &B) {
		let chunk_y = position.chunk_y() as usize;

		self.0[chunk_y].set_immediate(position.chunk(), target)
	}

	/// Makes sure that a future lookup for the target will succeed, unless the entry has changed since this call.
	pub fn ensure_available(&mut self, target: B) {
		for chunk in self.0.iter_mut() {
			chunk.ensure_available(target.clone());
		}
	}

	pub fn freeze_palettes(&mut self) -> (ColumnBlocks, ColumnPalettes<B>) {
		let chunks = slice_to_tuple_mut_16(self.0);

		let frozen = (
			chunks. 0.freeze_palette(), chunks. 1.freeze_palette(), chunks. 2.freeze_palette(), chunks. 3.freeze_palette(),
			chunks. 4.freeze_palette(), chunks. 5.freeze_palette(), chunks. 6.freeze_palette(), chunks. 7.freeze_palette(),
			chunks. 8.freeze_palette(), chunks. 9.freeze_palette(), chunks.10.freeze_palette(), chunks.11.freeze_palette(),
			chunks.12.freeze_palette(), chunks.13.freeze_palette(), chunks.14.freeze_palette(), chunks.15.freeze_palette()
		);

		(
			ColumnBlocks ([
				(frozen. 0).0, (frozen. 1).0, (frozen. 2).0, (frozen. 3).0,
				(frozen. 4).0, (frozen. 5).0, (frozen. 6).0, (frozen. 7).0,
				(frozen. 8).0, (frozen. 9).0, (frozen.10).0, (frozen.11).0,
				(frozen.12).0, (frozen.13).0, (frozen.14).0, (frozen.15).0
			]),
			ColumnPalettes([
				(frozen. 0).1, (frozen. 1).1, (frozen. 2).1, (frozen. 3).1,
				(frozen. 4).1, (frozen. 5).1, (frozen. 6).1, (frozen. 7).1,
				(frozen. 8).1, (frozen. 9).1, (frozen.10).1, (frozen.11).1,
				(frozen.12).1, (frozen.13).1, (frozen.14).1, (frozen.15).1
			])
		)
	}
}

#[derive(Debug)]
pub struct ColumnBlocks<'a>([&'a mut ChunkPacked; 16]);
impl<'a> ColumnBlocks<'a> {
	pub fn get<'p, B>(&self, at: ColumnPosition, palettes: &ColumnPalettes<'p, B>) -> Option<&'p B> where B: Target {
		let chunk_y = at.chunk_y() as usize;

		let raw = self.0[chunk_y].get(at.chunk());

		palettes.0[chunk_y].entries()[raw as usize].as_ref()
	}

	pub fn set(&mut self, at: ColumnPosition, association: &ColumnAssociation) {
		let chunk_y = at.chunk_y() as usize;

		self.0[chunk_y].set(at.chunk(), association.0[chunk_y])
	}
}

#[derive(Debug)]
pub struct ColumnPalettes<'a, B>([&'a Palette<B>; 16]) where B: 'a + Target;
impl<'a, B> ColumnPalettes<'a, B> where B: 'a + Target {
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Result<ColumnAssociation, usize> {
		Ok(ColumnAssociation ([
			self.0 [0].reverse_lookup(target).ok_or( 0usize)?,
			self.0 [1].reverse_lookup(target).ok_or( 1usize)?,
			self.0 [2].reverse_lookup(target).ok_or( 2usize)?,
			self.0 [3].reverse_lookup(target).ok_or( 3usize)?,
			self.0 [4].reverse_lookup(target).ok_or( 4usize)?,
			self.0 [5].reverse_lookup(target).ok_or( 5usize)?,
			self.0 [6].reverse_lookup(target).ok_or( 6usize)?,
			self.0 [7].reverse_lookup(target).ok_or( 7usize)?,
			self.0 [8].reverse_lookup(target).ok_or( 8usize)?,
			self.0 [9].reverse_lookup(target).ok_or( 9usize)?,
			self.0[10].reverse_lookup(target).ok_or(10usize)?,
			self.0[11].reverse_lookup(target).ok_or(11usize)?,
			self.0[12].reverse_lookup(target).ok_or(12usize)?,
			self.0[13].reverse_lookup(target).ok_or(13usize)?,
			self.0[14].reverse_lookup(target).ok_or(14usize)?,
			self.0[15].reverse_lookup(target).ok_or(15usize)?,
		]))
	}
}

#[derive(Debug)]
pub struct ColumnAssociation([u32; 16]);

pub fn slice_to_tuple_mut_16<T>(slice: &mut [T; 16])
								-> (&mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T, &mut T)
{
	let (s0 , slice) = slice.split_at_mut(1);
	let (s1 , slice) = slice.split_at_mut(1);
	let (s2 , slice) = slice.split_at_mut(1);
	let (s3 , slice) = slice.split_at_mut(1);
	let (s4 , slice) = slice.split_at_mut(1);
	let (s5 , slice) = slice.split_at_mut(1);
	let (s6 , slice) = slice.split_at_mut(1);
	let (s7 , slice) = slice.split_at_mut(1);
	let (s8 , slice) = slice.split_at_mut(1);
	let (s9 , slice) = slice.split_at_mut(1);
	let (s10, slice) = slice.split_at_mut(1);
	let (s11, slice) = slice.split_at_mut(1);
	let (s12, slice) = slice.split_at_mut(1);
	let (s13, slice) = slice.split_at_mut(1);
	let (s14, s15  ) = slice.split_at_mut(1);

	(
		&mut s0 [0], &mut s1 [0], &mut s2 [0], &mut s3 [0],
		&mut s4 [0], &mut s5 [0], &mut s6 [0], &mut s7 [0],
		&mut s8 [0], &mut s9 [0], &mut s10[0], &mut s11[0],
		&mut s12[0], &mut s13[0], &mut s14[0], &mut s15[0]
	)
}