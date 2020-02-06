mod system;

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::time::Instant;

fn main() -> Result<(), Error> {
    let rom_path = Path::new("./roms/pong.ch8");
    let rom_file = File::open(rom_path).unwrap();

    let mut reader = BufReader::new(rom_file);
    let mut buffer: Vec<u8> = vec!();

    reader.read_to_end(&mut buffer).unwrap();

    let mut test_system: system::System = system::System::new();
    test_system.write_rom(buffer);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();

    let window = {
        let size = LogicalSize::new(1024 as f64, 768 as f64);
        WindowBuilder::new()
            .with_title("tinyrenderer")
            .with_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut hidpi_factor = window.scale_factor();

    let mut pixels = {
        let surface = Surface::create(&window);
        let surface_texture = SurfaceTexture::new(64, 32, surface);
        Pixels::new(64, 32, surface_texture)?
    };

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            let previous_frame_time = last_frame;

            let frame = pixels.get_frame();

            pixels.render();

            last_frame = Instant::now();

            let delta = last_frame - previous_frame_time;

            let fps = (1.0 / ((delta.as_millis() as f64) / 1000.0)).round();

            window.set_title(&format!("CHIP-8 ({} fps)", fps));
        }

        if input.update(event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            window.request_redraw();
        }
    });

    while test_system.tick() {

    }

    Ok(())
}
