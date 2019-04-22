use std::time::Instant;
use std::fmt::Display;
use std::ops::AddAssign;
use vocs::position::GlobalSectorPosition;
use image::RgbImage;

pub mod full;

pub trait Renderer: Send {
	type SectorMetrics: Display + Default + Send + Sync + 'static;
	type TotalMetrics: TotalMetrics + AddAssign<Self::SectorMetrics>;

	fn process_sector(&self, sector_position: GlobalSectorPosition) -> (RgbImage, Self::SectorMetrics);
}

pub trait TotalMetrics: Display + Default {
	fn set_thread_count(&mut self, threads: u32);
	fn set_duration_us(&mut self, time: u64);
}

pub fn duration_us(start: &Instant) -> u64 {
	let end = Instant::now();
	let time = end.duration_since(*start);

	let secs = time.as_secs();

	(secs * 1000000) + ((time.subsec_nanos() / 1000) as u64)
}