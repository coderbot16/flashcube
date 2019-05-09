extern crate nbt_turbo;

use nbt_turbo::writer::CompoundWriter;

fn main() {
    let mut writer = CompoundWriter::start("hello world", Vec::new());

    writer
        .string("name", "Bananrama")
        .compound("inner")
            .i8("nested", 127);

    let buffer = writer.end();
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