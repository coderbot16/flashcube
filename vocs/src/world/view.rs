use world::chunk::{Target, Chunk, PaletteAssociation, Palette, NullRecorder};
use position::{ChunkPosition, ColumnPosition};
use storage::packed::PackedBlockStorage;

#[derive(Debug)]
pub struct ColumnMut<'c, B>(pub &'c mut [Chunk<B>; 16]) where B: 'c + Target;

impl<'c, B> ColumnMut<'c, B> where B: 'c + Target {
	pub fn get(&self, at: ColumnPosition) -> PaletteAssociation<B> {
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
pub struct ColumnBlocks<'a>([&'a mut PackedBlockStorage<ChunkPosition>; 16]);
impl<'a> ColumnBlocks<'a> {
	pub fn get<'p, B>(&self, at: ColumnPosition, palettes: &ColumnPalettes<'p, B>) -> PaletteAssociation<'p, B> where B: Target {
		let chunk_y = at.chunk_y() as usize;

		self.0[chunk_y].get(at.chunk(), palettes.0[chunk_y])
	}

	pub fn set<B>(&mut self, at: ColumnPosition, association: &ColumnAssociation<B>) where B: Target {
		let chunk_y = at.chunk_y() as usize;

		self.0[chunk_y].set(at.chunk(), &association.0[chunk_y], &mut NullRecorder)
	}
}

#[derive(Debug)]
pub struct ColumnPalettes<'a, B>([&'a Palette<B>; 16]) where B: 'a + Target;
impl<'a, B> ColumnPalettes<'a, B> where B: 'a + Target {
	/// Gets an association that will reference back to the target. Note that several indices may point to the same target, this returns one of them.
	pub fn reverse_lookup(&self, target: &B) -> Result<ColumnAssociation<B>, usize> {
		let palettes = slice_to_tuple_16(&self.0);

		Ok(ColumnAssociation ([
			palettes. 0.reverse_lookup(target).ok_or( 0usize)?,
			palettes. 1.reverse_lookup(target).ok_or( 1usize)?,
			palettes. 2.reverse_lookup(target).ok_or( 2usize)?,
			palettes. 3.reverse_lookup(target).ok_or( 3usize)?,
			palettes. 4.reverse_lookup(target).ok_or( 4usize)?,
			palettes. 5.reverse_lookup(target).ok_or( 5usize)?,
			palettes. 6.reverse_lookup(target).ok_or( 6usize)?,
			palettes. 7.reverse_lookup(target).ok_or( 7usize)?,
			palettes. 8.reverse_lookup(target).ok_or( 8usize)?,
			palettes. 9.reverse_lookup(target).ok_or( 9usize)?,
			palettes.10.reverse_lookup(target).ok_or(10usize)?,
			palettes.11.reverse_lookup(target).ok_or(11usize)?,
			palettes.12.reverse_lookup(target).ok_or(12usize)?,
			palettes.13.reverse_lookup(target).ok_or(13usize)?,
			palettes.14.reverse_lookup(target).ok_or(14usize)?,
			palettes.15.reverse_lookup(target).ok_or(15usize)?,
		]))
	}
}

#[derive(Debug)]
pub struct ColumnAssociation<'a, B>([PaletteAssociation<'a, B>; 16]) where B: 'a + Target;
impl<'a, B> ColumnAssociation<'a, B> where B: 'a + Target {
	pub fn raw_values(&self) -> [usize; 16] {
		let associations = slice_to_tuple_16(&self.0);

		[
			associations. 0.raw_value(),
			associations. 1.raw_value(),
			associations. 2.raw_value(),
			associations. 3.raw_value(),
			associations. 4.raw_value(),
			associations. 5.raw_value(),
			associations. 6.raw_value(),
			associations. 7.raw_value(),
			associations. 8.raw_value(),
			associations. 9.raw_value(),
			associations.10.raw_value(),
			associations.11.raw_value(),
			associations.12.raw_value(),
			associations.13.raw_value(),
			associations.14.raw_value(),
			associations.15.raw_value()
		]
	}
}

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

pub fn slice_to_tuple_16<T>(slice: &[T; 16])
							-> (&T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T, &T)
{
	let (s0 , slice) = slice.split_at(1);
	let (s1 , slice) = slice.split_at(1);
	let (s2 , slice) = slice.split_at(1);
	let (s3 , slice) = slice.split_at(1);
	let (s4 , slice) = slice.split_at(1);
	let (s5 , slice) = slice.split_at(1);
	let (s6 , slice) = slice.split_at(1);
	let (s7 , slice) = slice.split_at(1);
	let (s8 , slice) = slice.split_at(1);
	let (s9 , slice) = slice.split_at(1);
	let (s10, slice) = slice.split_at(1);
	let (s11, slice) = slice.split_at(1);
	let (s12, slice) = slice.split_at(1);
	let (s13, slice) = slice.split_at(1);
	let (s14, s15  ) = slice.split_at(1);

	(
		&s0 [0], &s1 [0], &s2 [0], &s3 [0],
		&s4 [0], &s5 [0], &s6 [0], &s7 [0],
		&s8 [0], &s9 [0], &s10[0], &s11[0],
		&s12[0], &s13[0], &s14[0], &s15[0]
	)
}