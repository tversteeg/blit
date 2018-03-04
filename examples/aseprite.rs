extern crate blit;
extern crate image;
extern crate minifb;
extern crate aseprite;
extern crate serde_json;

use blit::*;
use minifb::*;
use std::fs::File;

const WIDTH: usize = 250;
const HEIGHT: usize = 250;

const MASK_COLOR: u32 = 0xFF00FF;

fn main() {
    let mut buffer: Vec<u32> = vec![0x00FFFFFF; WIDTH * HEIGHT];

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Blit Animation Example - ESC to exit", WIDTH, HEIGHT, options).expect("Unable to open window");

    // Open the spritesheet image
    let img = image::open("examples/king-by-buch.png").unwrap();
    let rgba = img.as_rgba8().unwrap();
    // Convert it to a blitbuffer
    let blit_buf = rgba.to_blit_buffer(Color::from_u32(MASK_COLOR));

    // Open the spritesheet info
    let file = File::open("examples/king-by-buch.json").unwrap();
    let info: aseprite::SpritesheetData = serde_json::from_reader(file).unwrap();

    // Create the animation buffer object
    let anim = AnimationBlitBuffer::new(blit_buf, info);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        anim.blit_frame(&mut buffer, WIDTH, (0, 0), 0).unwrap();

        window.update_with_buffer(&buffer).unwrap();
    }
}
