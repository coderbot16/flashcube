extern crate nbt_turbo;

use nbt_turbo::writer::CompoundWriter;

fn main() {
	
	let buffer = CompoundWriter::write("hello world", Vec::new(), |writer| {
		writer.string("name", "Bananrama");
		writer.compound("inner",
			|writer| { 
				writer.i8("nested", 127);
			}
		);
	});

	dump_buffer(&buffer);
}

fn dump_buffer(buffer: &[u8]) {
	for (index, &value) in buffer.iter().enumerate() {
		if index != 0 && index % 16 == 0 {
			println!();
		}

		print!("{:02X} ", value);
	}
}