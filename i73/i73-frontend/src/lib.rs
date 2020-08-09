use i73_base::Block;
use i73_biome::{Biome, Followup, Lookup, Surface};
use std::collections::HashMap;

pub fn generate_biome_lookup() -> Lookup<Biome> {
	let mut biome_registry = HashMap::new();

	biome_registry.insert("swampland", Biome {
		name: "swampland".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("savanna", Biome {
		name: "savanna".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("plains", Biome {
		name: "plains".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("seasonal_forest", Biome {
		name: "seasonal_forest".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("desert", Biome {
		name: "desert".into(),
		surface: Surface {
			top: Block::SAND,
			fill: Block::SAND,
			chain: vec![
				Followup {
					block: Block::SANDSTONE,
					max_depth: 3
				}
			]
		}
	});

	biome_registry.insert("shrubland", Biome {
		name: "shrubland".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("taiga", Biome {
		name: "taiga".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("rainforest", Biome {
		name: "rainforest".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("ice_desert", Biome {
		name: "ice_desert".into(),
		surface: Surface {
			top: Block::SAND,
			fill: Block::SAND,
			chain: vec![
				Followup {
					block: Block::SANDSTONE,
					max_depth: 3
				}
			]
		}
	});

	biome_registry.insert("tundra", Biome {
		name: "tundra".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	biome_registry.insert("forest", Biome {
		name: "forest".into(),
		surface: Surface {
			top: Block::GRASS,
			fill: Block::DIRT,
			chain: vec![]
		}
	});

	let mut grid = i73_biome::Grid::new(biome_registry.get("plains").unwrap().clone());

	grid.add((0.00, 0.10), (0.00, 1.00), biome_registry.get("tundra").unwrap().clone());
	grid.add((0.10, 0.50), (0.00, 0.20), biome_registry.get("tundra").unwrap().clone());
	grid.add((0.10, 0.50), (0.20, 0.50), biome_registry.get("taiga").unwrap().clone());
	grid.add((0.10, 0.70), (0.50, 1.00), biome_registry.get("swampland").unwrap().clone());
	grid.add((0.50, 0.95), (0.00, 0.20), biome_registry.get("savanna").unwrap().clone());
	grid.add((0.50, 0.97), (0.20, 0.35), biome_registry.get("shrubland").unwrap().clone());
	grid.add((0.50, 0.97), (0.35, 0.50), biome_registry.get("forest").unwrap().clone());
	grid.add((0.70, 0.97), (0.50, 1.00), biome_registry.get("forest").unwrap().clone());
	grid.add((0.95, 1.00), (0.00, 0.20), biome_registry.get("desert").unwrap().clone());
	grid.add((0.97, 1.00), (0.20, 0.45), biome_registry.get("plains").unwrap().clone());
	grid.add((0.97, 1.00), (0.45, 0.90), biome_registry.get("seasonal_forest").unwrap().clone());
	grid.add((0.97, 1.00), (0.90, 1.00), biome_registry.get("rainforest").unwrap().clone());

	let grid = grid;

	Lookup::generate(&grid)
}
