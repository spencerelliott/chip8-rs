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
        let color = if byte > &0 { 255 } else { 0 };

        p3_file.write_fmt(format_args!(
            "{0} {0} {0}\n",
            color
        ))?;
    }

    println!("Hello, world!");

    Ok(())
}
