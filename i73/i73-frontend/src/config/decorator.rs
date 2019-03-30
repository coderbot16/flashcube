use serde_json;
use decorator::Decorator;
use vocs::indexed::Target;

pub trait DecoratorFactory<B> where B: Target {
	fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator<B>>>;
}

/// Vein decorator factories
pub mod vein {
	use super::*;
	use decorator::vein::{VeinDecorator, SeasideVeinDecorator};

	#[derive(Default)]
	pub struct VeinDecoratorFactory<B>(::std::marker::PhantomData<B>);
	impl<B> DecoratorFactory<B> for VeinDecoratorFactory<B> where B: 'static + Target + ::serde::Deserialize {
		fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator<B>>> {
			Ok(Box::new(serde_json::from_value::<VeinDecorator<B>>(config)?))
		}
	}

	#[derive(Default)]
	pub struct SeasideVeinDecoratorFactory<B>(::std::marker::PhantomData<B>);
	impl<B> DecoratorFactory<B> for SeasideVeinDecoratorFactory<B> where B: 'static + Target + ::serde::Deserialize {
		fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator<B>>> {
			Ok(Box::new(serde_json::from_value::<SeasideVeinDecorator<B>>(config)?))
		}
	}
}

/// Lake decorator factories
pub mod lake {
	use super::*;
	use decorator::lake::{LakeDecorator};

	#[derive(Default)]
	pub struct LakeDecoratorFactory<B>(::std::marker::PhantomData<B>);
	impl<B> DecoratorFactory<B> for LakeDecoratorFactory<B> where B: 'static + Target + ::serde::Deserialize {
		fn configure(&self, config: serde_json::Value) -> serde_json::Result<Box<Decorator<B>>> {
			Ok(Box::new(serde_json::from_value::<LakeDecorator<B>>(config)?))
		}
	}
}