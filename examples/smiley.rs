extern crate blit;
extern crate image;
extern crate minifb;

use blit::*;
use image::GenericImage;
use minifb::*;

const WIDTH: usize = 180;
const HEIGHT: usize = 180;

fn main() {
    let mut buffer: Vec<u32> = vec![0x00FFFFFF; WIDTH * HEIGHT];

    let img = image::open("examples/smiley.png").unwrap();
    println!("Loaded image with size {:?}", img.dimensions());

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Blit Example - ESC to exit & click to draw", WIDTH, HEIGHT, options).expect("Unable to open window");

    let img_size = img.dimensions();

    let rgb = img.as_rgb8().unwrap();
    rgb.blit_with_mask_color(&mut buffer, (WIDTH, HEIGHT), (0, 0), 0xFFFFFF);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            if window.get_mouse_down(MouseButton::Left) {
                let x_pos = mouse.0 as i32 - (img_size.0 / 2) as i32;
                let y_pos = mouse.1 as i32 - (img_size.1 / 2) as i32;
                rgb.blit_with_mask_color(&mut buffer, (WIDTH, HEIGHT), (x_pos, y_pos), 0x00FF00FF);
            }
        });

        window.update_with_buffer(&buffer).unwrap();
    }
}
