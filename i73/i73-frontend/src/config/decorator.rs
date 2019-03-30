use serde_json;
use i73_decorator::Decorator;

pub trait DecoratorFactory {
	fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator>>;
}

/// Vein decorator factories
pub mod vein {
	use super::*;
	use i73_decorator::vein::{VeinDecorator, SeasideVeinDecorator};

	#[derive(Default)]
	pub struct VeinDecoratorFactory;
	impl DecoratorFactory for VeinDecoratorFactory {
		fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator>> {
			Ok(Box::new(serde_json::from_value::<VeinDecorator>(config)?))
		}
	}

	#[derive(Default)]
	pub struct SeasideVeinDecoratorFactory;
	impl DecoratorFactory for SeasideVeinDecoratorFactory {
		fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator>> {
			Ok(Box::new(serde_json::from_value::<SeasideVeinDecorator>(config)?))
		}
	}
}

/// Lake decorator factories
pub mod lake {
	use super::*;
	use i73_decorator::lake::{LakeDecorator};

	#[derive(Default)]
	pub struct LakeDecoratorFactory;
	impl DecoratorFactory for LakeDecoratorFactory {
		fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator>> {
			Ok(Box::new(serde_json::from_value::<LakeDecorator>(config)?))
		}
	}
}