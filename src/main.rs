mod system;
mod ops;

use std::fs::File;
use std::io::{prelude::*, BufReader, Write};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let rom_path = Path::new("./roms/zero.ch8");
    let rom_file = File::open(rom_path).unwrap();

    let mut reader = BufReader::new(rom_file);
    let mut buffer: Vec<u8> = vec!();

    reader.read_to_end(&mut buffer)?;

    let mut test_system: system::System = system::System::new();
    test_system.write_rom(buffer);

    for _ in 0..100 {
        test_system.tick();
    }

    println!("Hello, world!");

    Ok(())
}
