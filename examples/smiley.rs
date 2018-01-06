extern crate blit;
extern crate image;
extern crate minifb;

use blit::*;
use image::GenericImage;
use minifb::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let img = image::open("examples/smiley.png").unwrap();
    println!("Loaded image with size {:?}", img.dimensions());

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Blit Smiley Example - Press ESC to exit", WIDTH, HEIGHT, options).expect("Unable to open window");

    img.blit_on_buffer(&mut buffer, WIDTH, (0, 0), 0x00FF00FF);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&buffer).unwrap();
    }
}
