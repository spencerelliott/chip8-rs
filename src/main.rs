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

    println!("Hello, world!");

    Ok(())
}
