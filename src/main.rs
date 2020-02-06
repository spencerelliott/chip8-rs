mod system;

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let rom_path = Path::new("./roms/pong.ch8");
    let rom_file = File::open(rom_path).unwrap();

    let mut reader = BufReader::new(rom_file);
    let mut buffer: Vec<u8> = vec!();

    reader.read_to_end(&mut buffer)?;

    let mut test_system: system::System = system::System::new();
    test_system.write_rom(buffer);

    while test_system.tick() {

    }

    let mut p3_file = File::create("out.ppm")?;
    p3_file.write_fmt(format_args!("P3\n{} {}\n255\n", 64, 32))?;

    for byte in test_system.get_framebuffer() {
        let colors = [
            if byte & 0b1000_0000 > 0 { 255 } else { 0 },
            if byte & 0b0100_0000 > 0 { 255 } else { 0 },
            if byte & 0b0010_0000 > 0 { 255 } else { 0 },
            if byte & 0b0001_0000 > 0 { 255 } else { 0 },
            if byte & 0b0000_1000 > 0 { 255 } else { 0 },
            if byte & 0b0000_0100 > 0 { 255 } else { 0 },
            if byte & 0b0000_0010 > 0 { 255 } else { 0 },
            if byte & 0b0000_0001 > 0 { 255 } else { 0 },
        ];

        for color in colors.iter() {
            p3_file.write_fmt(format_args!(
                "{0} {0} {0}\n",
                color
            ))?;
        }
    }

    println!("Hello, world!");

    Ok(())
}
