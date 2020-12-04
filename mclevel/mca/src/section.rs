use nbt_turbo::writer::{CompoundWriter, Output};
use vocs::indexed::{IndexedCube, Target};
use vocs::nibbles::{u4, NibbleCube};
use vocs::position::CubePosition;

// TODO: Cannot derive Debug (array of length 4096)
#[derive(Clone)]
pub struct SectionRef<'c> {
	pub y: i8,
	pub blocks: &'c [u8; 4096],
	pub add: Option<&'c NibbleCube>,
	pub data: &'c NibbleCube,
	pub block_light: &'c NibbleCube,
	pub sky_light: &'c NibbleCube,
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
	pub add: Option<NibbleCube>,
	pub data: NibbleCube,
	pub block_light: NibbleCube,
	pub sky_light: NibbleCube,
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
	pub add: Option<NibbleCube>,
	pub data: NibbleCube
}

impl AnvilBlocks {
	pub fn empty() -> Self {
		AnvilBlocks {
			blocks: Box::new([0u8; 4096]),
			data: NibbleCube::default(),
			add: None
		}
	}

	pub fn from_paletted<'b, B, F>(chunk: &'b IndexedCube<B>, to_anvil_id: &'b F) -> Option<Self> 
	where B: 'b + Target, F: Fn(&'b B) -> u16, {
		let mut blocks = Box::new([0u8; 4096]);
		let mut meta = NibbleCube::default();
	
		let (storage, palette) = chunk.freeze();
	
		// Can't express Anvil IDs over 4095 without Add. TODO: Utilize Counts.
		let need_add = palette.iter().map(|slot| slot.as_ref().map(to_anvil_id).unwrap_or(0)).any(|id| id > 4095);
	
		let mut has_any = false;

		let add = if need_add {
			let mut add = NibbleCube::default();
	
			for position in CubePosition::enumerate() {
				let raw = storage.get(position);
				let anvil = to_anvil_id(palette[raw as usize].as_ref().unwrap());

				if anvil != 0 {
					has_any = true
				}
	
				blocks[position.yzx() as usize] = (anvil >> 4) as u8;
				meta.set_uncleared(position, u4::new((anvil & 0xF) as u8));
				add.set_uncleared(position, u4::new((anvil >> 12) as u8));
			}
	
			Some(add)
		} else {
			for position in CubePosition::enumerate() {
				let raw = storage.get(position);
				let anvil = to_anvil_id(palette[raw as usize].as_ref().unwrap());
	
				if anvil != 0 {
					has_any = true
				}

				blocks[position.yzx() as usize] = (anvil >> 4) as u8;
				meta.set_uncleared(position, u4::new((anvil & 0xF) as u8));
			}
	
			None
		};

		if !has_any {
			return None
		}

		Some(AnvilBlocks {
				blocks,
				data: meta,
				add
		})
	}
}
