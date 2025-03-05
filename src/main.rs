#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use game::{app::App, World};
use iron_oxide::ui::{UiType, ErasedFnPointer};
use winit::event_loop::EventLoop;

mod graphic;
mod game;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut application = App::run();

    {
        let mut ui = application.ui.borrow_mut();
        let element = unsafe { ui.get_element_mut(vec![2]).unwrap() };

        if let UiType::Button(button) = &mut element.inherit {
            button.on_press = ErasedFnPointer::from_associated_ui(&mut application.world, World::restart);
        }
    };

    event_loop.run_app(&mut application).unwrap();
    drop(application.renderer)
}

#[allow(unused_assignments)]
#[test]
fn test() {
    use std::{fs::File, io::Write, path::Path};
    use png::Decoder;

    let decoder = Decoder::new(File::open(Path::new("C:/Dev/vudeljump/font/default8.png")).unwrap());
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();

    let mut f = File::create("C:/Dev/vudeljump/font/std1.fef").unwrap();
    let mut buf2 = Vec::with_capacity(2048);

    let image_size = info.width;
    let chars_in_row = 16;
    let char_i = 32;
    let char_end = 128;
    let char_space = image_size / chars_in_row;

    for i in char_i..char_end {
        let x = i % chars_in_row;
        let y = i / chars_in_row;
        let data_offset = y * image_size * char_space + x * char_space;
        let mut start: (u16, u16) = (0, 16);
        let mut length: u16 = 0;
        let mut height: u16 = 0;

        let mut found_first_pixel = false;
        let mut min_x: u16 = char_space as u16;
        let mut max_x: u16 = 0;
        let mut max_y: u16 = 0;

        for i_y in 0..char_space {
            for i_x in 0..char_space {
                let data = buf[(data_offset + i_x + i_y * image_size) as usize];
                if data != 0 {
                    if !found_first_pixel {
                        // Setze Startpunkt beim ersten gefundenen Pixel
                        start = ((i_x + x * char_space) as u16, (y * char_space) as u16);
                        found_first_pixel = true;
                    }
                    // Update min_x, max_x, und max_y für jedes gefundene Pixel
                    if (i_x as u16) < min_x {
                        min_x = i_x as u16;
                    }
                    if (i_x as u16) > max_x {
                        max_x = i_x as u16;
                    }
                    if (i_y as u16) > max_y {
                        max_y = i_y as u16;
                    }
                }
            }
        }

        // Breite und Höhe berechnen, aber nur wenn Pixel gefunden wurden
        if found_first_pixel {
            length = max_x - min_x + 2;
            height = max_y + 1 - (start.1 % char_space as u16);
            start.0 = min_x + x as u16 * char_space as u16; // Setze start.0 auf min_x
        } else {
            // Wenn kein Pixel gefunden wurde, setze Länge und Höhe auf 0
            length = 2;
            height = 7;
        }

        let end = (length, height);

        buf2.extend_from_slice(&start.0.to_le_bytes());
        buf2.extend_from_slice(&start.1.to_le_bytes());
        buf2.extend_from_slice(&end.0.to_le_bytes());
        buf2.extend_from_slice(&end.1.to_le_bytes());

        println!("Char: {}, Start: {:?}, Length: {}, Height: {}", i, start, length, height);
    }

    f.write(&buf2).unwrap();
}
