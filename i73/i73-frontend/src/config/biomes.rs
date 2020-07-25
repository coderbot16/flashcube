use i73_base::Block;
use i73_biome::{Biome, Followup, Grid, Surface};
use std::borrow::Cow;
use std::collections::HashMap;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum Error {
	ParseInt(ParseIntError),
	UnknownBiome(String),
}

impl From<ParseIntError> for Error {
	fn from(from: ParseIntError) -> Self {
		Error::ParseInt(from)
	}
}

#[derive(Debug)]
pub struct BiomesConfig {
	pub biomes: HashMap<String, BiomeConfig>,
	pub default: String,
	pub grid: Vec<RectConfig>,
}

impl BiomesConfig {
	pub fn to_grid(&self) -> Result<Grid<Biome>, Error> {
		let mut translated = HashMap::with_capacity(self.biomes.capacity());

		for (name, biome) in &self.biomes {
			translated.insert(name.clone(), biome.to_biome()?);
		}

		let default = translated
			.get(&self.default)
			.ok_or_else(|| Error::UnknownBiome(self.default.clone()))?;

		let mut grid = Grid::new(default.clone());

		for rect in &self.grid {
			let biome = translated
				.get(&rect.biome)
				.ok_or_else(|| Error::UnknownBiome(rect.biome.clone()))?;

			grid.add(rect.temperature, rect.rainfall, biome.clone());
		}

		Ok(grid)
	}
}

#[derive(Debug)]
pub struct BiomeConfig {
	pub debug_name: String,
	pub surface: SurfaceConfig,
	pub decorators: Vec<String>,
}

impl BiomeConfig {
	pub fn to_biome(&self) -> Result<Biome, ParseIntError> {
		Ok(Biome { name: Cow::Owned(self.debug_name.clone()), surface: self.surface.to_surface()? })
	}
}

#[derive(Debug)]
pub struct SurfaceConfig {
	pub top: String,
	pub fill: String,
	pub chain: Vec<FollowupConfig>,
}

impl SurfaceConfig {
	pub fn to_surface(&self) -> Result<Surface, ParseIntError> {
		Ok(Surface {
			top: parse_id(&self.top)?,
			fill: parse_id(&self.fill)?,
			chain: self
				.chain
				.iter()
				.map(FollowupConfig::to_followup)
				.collect::<Result<Vec<Followup>, ParseIntError>>()?,
		})
	}
}

#[derive(Debug)]
pub struct FollowupConfig {
	pub block: String,
	pub max_depth: u32,
}

impl FollowupConfig {
	pub fn to_followup(&self) -> Result<Followup, ParseIntError> {
		Ok(Followup { block: parse_id(&self.block)?, max_depth: self.max_depth })
	}
}

#[derive(Debug)]
pub struct RectConfig {
	pub temperature: (f64, f64),
	pub rainfall: (f64, f64),
	pub biome: String,
}

pub fn parse_id(id: &str) -> Result<Block, ParseIntError> {
	let mut split = id.split(':');

	let primary = split.next().unwrap().parse::<u16>()?;
	let secondary = split.next().map(|s| s.parse::<u16>());

	let secondary = match secondary {
		Some(secondary) => secondary?,
		None => 0,
	};

	Ok(Block::from_anvil_id(primary * 16 + secondary))
}
