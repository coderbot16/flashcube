use image::RgbImage;
use std::fs;
use std::thread;
use std::cmp;
use std::sync::mpsc;

use crate::renderer::{duration_us, Renderer, TotalMetrics};
use vocs::position::GlobalSectorPosition;
use std::path::Path;

pub fn generate_stitched_image<F, R>(create_renderer: F, name: String, sector_size: (u32, u32), offset: (u32, u32), thread_count: u32, quiet: bool) where F: 'static + Send + Sync + Fn() -> R, R: 'static + Renderer + Sync {
	println!("[=======] Generating {} map...", name);
	let gen_start = ::std::time::Instant::now();
	let mut map = RgbImage::new(sector_size.0 * 256, sector_size.1 * 256);

	let sector_count = sector_size.0 * sector_size.1;
	let per_sector = sector_count / thread_count;
	let mut threads = Vec::with_capacity(thread_count as usize);

	let (sender, receiver) = mpsc::channel();
	let mut total_metrics = R::TotalMetrics::default();
	total_metrics.set_thread_count(thread_count);

	let mut sector_allotments = sector_count;
	for _ in 0..thread_count {
		let base = sector_size.0 * sector_size.1 - sector_allotments;
		let allotment = cmp::min(per_sector, sector_allotments);
		sector_allotments -= allotment;
		let sender = sender.clone();
		let renderer = create_renderer();

		let handle = thread::spawn(move || {
			for index in 0..allotment {
				let index = index + base;
				let (x, z) = (index % sector_size.0, index / sector_size.0);

				let position = GlobalSectorPosition::new(
					x as i32 + offset.0 as i32,
					z as i32 + offset.1 as i32
				);

				let (sector, metrics) = renderer.process_sector(position);

				sender.send((x, z, sector, metrics)).unwrap();
			}
		});

		threads.push(handle);
	}

	let mut recieved = 0;
	let mut last_percentage = 0;

	while let Ok((x, z, sector, metrics)) = receiver.recv() {
		for iz in 0..256 {
			for ix in 0..256 {
				map.put_pixel(
					x * 256 + ix,
					z * 256 + iz,
					*sector.get_pixel(ix, iz)
				);
			}
		}

		recieved += 1;

		let percentage = (recieved as f64 / sector_count as f64) * 100.0;
		if percentage as u32 > last_percentage || !quiet {
			last_percentage = percentage as u32;
			println!("[{:6.2}%] Sector: ({:2}, {:2}) | {}", percentage, x, z, metrics);
		}

		total_metrics += metrics;

		if recieved >= sector_count {
			break;
		}
	}

	for thread in threads {
		thread.join().unwrap();
	}

	let total = duration_us(&gen_start);
	total_metrics.set_duration_us(total);

	println!("[=======] Generation complete in {:.3}ms, {:.3}ms/sector, {:.3}ms/column | {}",
			 (total as f64) / 1000.0,
			 (total * thread_count as u64 / (sector_count as u64)) as f64 / 1000.0,
			 (total * thread_count as u64/ (sector_count as u64 * 256)) as f64 / 1000.0,
			 total_metrics
	);

	println!("[=======] Saving image...");

	if let Some(parent) = Path::new(&name).parent() {
		fs::create_dir_all(parent).unwrap();
	}

	map.save(name).unwrap();
	let us = duration_us(&gen_start) - total;

	println!("[=======] Saving complete in {:.3}ms", (us as f64) / 1000.0);
}