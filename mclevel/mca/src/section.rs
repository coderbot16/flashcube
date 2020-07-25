use nbt_turbo::writer::{CompoundWriter, Output};
use vocs::indexed::{ChunkIndexed, Target};
use vocs::nibbles::{u4, ChunkNibbles};
use vocs::position::ChunkPosition;

// TODO: Cannot derive Debug (array of length 4096)
#[derive(Clone)]
pub struct SectionRef<'c> {
	pub y: i8,
	pub blocks: &'c [u8; 4096],
	pub add: Option<&'c ChunkNibbles>,
	pub data: &'c ChunkNibbles,
	pub block_light: &'c ChunkNibbles,
	pub sky_light: &'c ChunkNibbles,
}

impl<'c> SectionRef<'c> {
	pub fn write(&self, writer: &mut CompoundWriter<impl Output>) {
		writer
			.i8("Y", self.y)
			.u8_array("Blocks", &self.blocks[..])
			.u8_array("Data", self.data.raw())
			.u8_array("BlockLight", self.block_light.raw())
			.u8_array("SkyLight", self.sky_light.raw());

		if let Some(add) = self.add {
			writer.u8_array("Add", add.raw());
		}
	}
}

#[derive(Clone)]
pub struct Section {
	pub y: i8,
	pub blocks: Box<[u8; 4096]>,
	pub add: Option<ChunkNibbles>,
	pub data: ChunkNibbles,
	pub block_light: ChunkNibbles,
	pub sky_light: ChunkNibbles,
}

impl Section {
	pub fn to_ref(&self) -> SectionRef {
		SectionRef {
			y: self.y,
			blocks: &self.blocks,
			add: self.add.as_ref(),
			data: &self.data,
			block_light: &self.block_light,
			sky_light: &self.sky_light
		}
	}
}

pub struct AnvilBlocks {
	pub blocks: Box<[u8; 4096]>,
	pub add: Option<ChunkNibbles>,
	pub data: ChunkNibbles
}

impl AnvilBlocks {
	pub fn from_paletted<'b, B, F>(chunk: &'b ChunkIndexed<B>, to_anvil_id: &'b F) -> Self 
	where B: 'b + Target, F: Fn(&'b B) -> u16, {
		let mut blocks = Box::new([0u8; 4096]);
		let mut meta = ChunkNibbles::default();
	
		let (storage, palette) = chunk.freeze();
	
		// Can't express Anvil IDs over 4095 without Add. TODO: Utilize Counts.
		let need_add = palette.iter().map(|slot| slot.as_ref().map(to_anvil_id).unwrap_or(0)).any(|id| id > 4095);
	
		let add = if need_add {
			let mut add = ChunkNibbles::default();
	
			for position in ChunkPosition::enumerate() {
				let raw = storage.get(position);
				let anvil = to_anvil_id(palette[raw as usize].as_ref().unwrap());
	
				blocks[position.yzx() as usize] = (anvil >> 4) as u8;
				meta.set_uncleared(position, u4::new((anvil & 0xF) as u8));
				add.set_uncleared(position, u4::new((anvil >> 12) as u8));
			}
	
			Some(add)
		} else {
			for position in ChunkPosition::enumerate() {
				let raw = storage.get(position);
				let anvil = to_anvil_id(palette[raw as usize].as_ref().unwrap());
	
				blocks[position.yzx() as usize] = (anvil >> 4) as u8;
				meta.set_uncleared(position, u4::new((anvil & 0xF) as u8));
			}
	
			None
		};
	
		AnvilBlocks {
				blocks,
				data: meta,
				add
		}
	}
}
