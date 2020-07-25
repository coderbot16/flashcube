use nbt_turbo::writer::{CompoundWriter, Output};

mod section;

pub use section::{AnvilBlocks, Section, SectionRef};

// TODO: Cannot derive Debug (Column)
#[derive(Clone)]
pub struct ColumnRoot<'c> {
	/// Patch version of the NBT structure.
	///
	/// Determines the version of the schema used by DataFixerUpper.
	/// Columns missing a data version are assumed to be from 15w31c or below (ie, 1.8.9 or below)
	pub version: Option<i32>,
	pub column: Column<'c>,
}

impl<'c> ColumnRoot<'c> {
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

impl<'c> From<Column<'c>> for ColumnRoot<'c> {
	fn from(column: Column<'c>) -> Self {
		ColumnRoot { version: None, column }
	}
}

// TODO: Cannot derive Debug (Section)
#[derive(Clone)]
pub struct Column<'c> {
	pub x: i32,
	pub z: i32,
	pub last_update: i64,
	pub light_populated: bool,
	pub terrain_populated: bool,
	pub v: Option<i8>,
	pub inhabited_time: i64,
	pub biomes: &'c [u8],
	pub heightmap: &'c [u32],
	pub sections: &'c [SectionRef<'c>],
	pub tile_ticks: &'c [ScheduledTick], // TODO: Entities, TileEntities
}

impl<'c> Column<'c> {
	pub fn write(&self, writer: &mut CompoundWriter<impl Output>) {
		writer
			.i32("xPos", self.x)
			.i32("zPos", self.z)
			.i64("LastUpdate", self.last_update)
			.bool("LightPopulated", self.light_populated)
			.bool("TerrainPopulated", self.terrain_populated)
			.i64("InhabitedTime", self.inhabited_time)
			.u8_array("Biomes", self.biomes)
			.u32_array("HeightMap", self.heightmap)
			.compound_array("Sections", self.sections.len(), |sections| {
				for section in self.sections {
					sections.compound(|writer| {
						section.write(writer);
					});
				}
			})
			.compound_array("TileTicks", 0, |ticks| {
				for tick in self.tile_ticks {
					ticks.compound(|writer| {
						tick.write(writer);
					});
				}
			})
			.compound_array("Entities", 0, |_| {
				todo!("cannot write entities");
			})
			.compound_array("TileEntities", 0, |_| {
				todo!("cannot write tile entities");
			});

		if let Some(&v) = self.v.as_ref() {
			writer.i8("V", v);
		}
	}
}

impl<'c> From<ColumnRoot<'c>> for Column<'c> {
	fn from(root: ColumnRoot<'c>) -> Self {
		root.column
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
