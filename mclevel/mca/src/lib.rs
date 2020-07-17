use nbt_turbo::writer::{CompoundWriter, Output};
use vocs::nibbles::ChunkNibbles;

#[derive(Debug, Clone)]
pub struct ColumnRoot {
	/// Patch version of the NBT structure.
	///
	/// Determines the version of the schema used by DataFixerUpper.
	/// Columns missing a data version are assumed to be from 15w31c or below (ie, 1.8.9 or below)
	pub version: Option<i32>,
	pub column: Column,
}

impl ColumnRoot {
	pub fn write<T: Output>(&self, out: T) -> T {
		CompoundWriter::write("", out, |writer| {
			if let Some(version) = self.version {
				writer.i32("DataVersion", version);
			}

			writer.compound("Level", |writer| {
				self.column.write(writer);
			})
		})
	}
}

impl From<Column> for ColumnRoot {
	fn from(column: Column) -> Self {
		ColumnRoot { version: Some(0), column }
	}
}

#[derive(Debug, Clone)]
pub struct Column {
	pub x: i32,
	pub z: i32,
	pub last_update: i64,
	pub light_populated: bool,
	pub terrain_populated: bool,
	pub v: Option<i8>,
	pub inhabited_time: i64,
	pub biomes: Vec<u8>,
	pub heightmap: Vec<u32>,
	pub sections: Vec<Section>,
	pub tile_ticks: Vec<ScheduledTick>, // TODO: Entities, TileEntities
}

impl Column {
	pub fn write(&self, writer: &mut CompoundWriter<impl Output>) {
		writer
			.i32("xPos", self.x)
			.i32("zPos", self.z)
			.i64("LastUpdate", self.last_update)
			.bool("LightPopulated", self.light_populated)
			.bool("TerrainPopulated", self.terrain_populated)
			.i64("InhabitedTime", self.inhabited_time)
			.u8_array("Biomes", &self.biomes)
			.u32_array("HeightMap", &self.heightmap)
			.compound_array("Sections", self.sections.len(), |sections| {
				for section in &self.sections {
					sections.compound(|writer| {
						section.write(writer);
					});
				}
			})
			.compound_array("TileTicks", 0, |ticks| {
				for tick in &self.tile_ticks {
					ticks.compound(|writer| {
						tick.write(writer);
					});
				}
			})
			.compound_array("Entities", 0, |_| {
				// TODO: Entities
			})
			.compound_array("TileEntities", 0, |_| {
				// TODO: TileEntities
			});

		if let Some(&v) = self.v.as_ref() {
			writer.i8("V", v);
		}
	}
}

impl From<ColumnRoot> for Column {
	fn from(root: ColumnRoot) -> Self {
		root.column
	}
}

#[derive(Debug, Clone)]
pub struct Section {
	pub y: i8,
	pub blocks: Vec<u8>,
	pub add: Option<ChunkNibbles>,
	pub data: ChunkNibbles,
	pub block_light: ChunkNibbles,
	pub sky_light: ChunkNibbles,
}

impl Section {
	pub fn write(&self, writer: &mut CompoundWriter<impl Output>) {
		writer
			.i8("Y", self.y)
			.u8_array("Blocks", &self.blocks)
			.u8_array("Data", self.data.raw())
			.u8_array("BlockLight", self.block_light.raw())
			.u8_array("SkyLight", self.sky_light.raw());

		if let Some(add) = self.add.as_ref() {
			writer.u8_array("Add", add.raw());
		}
	}
}

#[derive(Debug, Clone)]
pub struct ScheduledTick {
	pub id: String,
	pub delay: i32,
	pub priority: i32,
	pub x: i32,
	pub y: i32,
	pub z: i32,
}

impl ScheduledTick {
	pub fn write(&self, writer: &mut CompoundWriter<impl Output>) {
		writer
			.string("i", &self.id)
			.i32("t", self.delay)
			.i32("p", self.priority)
			.i32("x", self.x)
			.i32("y", self.y)
			.i32("z", self.z);
	}
}
