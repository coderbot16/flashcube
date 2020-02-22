use std::time::Instant;
use std::fmt::{self, Display};
use std::ops::AddAssign;
use vocs::position::GlobalSectorPosition;
use image::RgbImage;

pub mod full;
pub mod climate;

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

#[derive(Default)]
pub struct BasicTimeMetrics {
	pub total: u64
}

impl Display for BasicTimeMetrics {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:8.3}ms, {:5.3}ms/column",
			   (self.total as f64) / 1000.0,
			   (self.total / 256) as f64 / 1000.0
		)
	}
}

#[derive(Default)]
pub struct BasicTotalMetrics {
	pub total: u64,
	pub thread_count: u32
}

impl TotalMetrics for BasicTotalMetrics {
	fn set_thread_count(&mut self, threads: u32) {
		self.thread_count = threads;
	}

	fn set_duration_us(&mut self, time: u64) {
		self.total = time;
	}
}

impl AddAssign<BasicTimeMetrics> for BasicTotalMetrics {
	fn add_assign(&mut self, _other: BasicTimeMetrics) {}
}

impl Display for BasicTotalMetrics {
	fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
		Ok(())
	}
}