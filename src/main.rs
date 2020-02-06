mod system;

use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::path::Path;
use std::thread;

use pixels::{wgpu::Surface, Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
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
        let size = LogicalSize::new(256 as f64, 128 as f64);
        WindowBuilder::new()
            .with_title("CHIP-8")
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
    let mut last_tick = Instant::now();
    let frame_duration = std::time::Duration::from_secs_f32(1.0/60.0);
    let tick_duration = std::time::Duration::from_secs_f32(1.0/600.0);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::RedrawRequested(_) => {
                let previous_frame_time = last_frame;
    
                let mut frame = pixels.get_frame();
                let framebuffer = test_system.get_framebuffer();
                frame.write(framebuffer).unwrap();
                pixels.render();

                last_frame = Instant::now();
    
                let delta = last_frame - previous_frame_time;
                let fps = (1.0 / ((delta.as_millis() as f64) / 1000.0)).round();
            
                window.set_title(&format!("CHIP-8 ({} fps)", fps));
            }
            _ => {}
        }
        

        if input.update(event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            test_system.set_key(0x0, input.key_held(VirtualKeyCode::Key0));
            test_system.set_key(0x1, input.key_held(VirtualKeyCode::Key1));
            test_system.set_key(0x2, input.key_held(VirtualKeyCode::Key2));
            test_system.set_key(0x3, input.key_held(VirtualKeyCode::Key3));
            test_system.set_key(0x4, input.key_held(VirtualKeyCode::Key4));
            test_system.set_key(0x5, input.key_held(VirtualKeyCode::Key5));
            test_system.set_key(0x6, input.key_held(VirtualKeyCode::Key6));
            test_system.set_key(0x7, input.key_held(VirtualKeyCode::Key7));
            test_system.set_key(0x8, input.key_held(VirtualKeyCode::Key8));
            test_system.set_key(0x9, input.key_held(VirtualKeyCode::Key9));
            test_system.set_key(0xA, input.key_held(VirtualKeyCode::A));
            test_system.set_key(0xB, input.key_held(VirtualKeyCode::B));
            test_system.set_key(0xC, input.key_held(VirtualKeyCode::C));
            test_system.set_key(0xD, input.key_held(VirtualKeyCode::D));
            test_system.set_key(0xE, input.key_held(VirtualKeyCode::E));
            test_system.set_key(0xF, input.key_held(VirtualKeyCode::F));

            if let Some(factor) = input.scale_factor_changed() {
                hidpi_factor = factor;
            }

            let delta = Instant::now() - last_tick;

            if delta < tick_duration {
                thread::sleep(tick_duration - delta);
            }

            if last_tick.elapsed() > tick_duration {
                test_system.tick();
                last_tick = Instant::now();
            }

            if last_frame.elapsed() > frame_duration {
                window.request_redraw();
            }
        }
    });
}
