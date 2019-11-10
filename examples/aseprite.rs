#[cfg(all(feature = "aseprite", feature = "image"))]
extern crate aseprite;
extern crate blit;
extern crate image;
extern crate minifb;
extern crate serde_json;

use blit::*;
use minifb::*;
use std::fs::File;
use std::time::SystemTime;

const WIDTH: usize = 512;
const HEIGHT: usize = 120;

const MASK_COLOR: u32 = 0xFF00FF;

#[cfg(all(feature = "aseprite", feature = "image"))]
fn main() {
    let mut buffer: Vec<u32> = vec![0x00FFFFFF; WIDTH * HEIGHT];

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new(
        "Blit Animation Example - ESC to exit",
        WIDTH,
        HEIGHT,
        options,
    )
    .expect("Unable to open window");

    // Open the spritesheet image
    let img = image::open("examples/king-by-buch.png").unwrap();
    let rgba = img.as_rgba8().unwrap();
    // Convert it to a blitbuffer
    let blit_buf = rgba.to_blit_buffer(Color::from_u32(MASK_COLOR));

    // Open the spritesheet info
    let file = File::open("examples/king-by-buch.json").unwrap();
    let info: aseprite::SpritesheetData = serde_json::from_reader(file).unwrap();

    // Create the animation buffer object
    let anim_buffer = AnimationBlitBuffer::new(blit_buf, info);

    // Create the animations using hardcoded frames
    let mut walk_anim = Animation::start(0, 3, true);
    let mut jump_anim = Animation::start(4, 7, true);
    let mut full_anim = Animation::start(0, 11, true);

    // Create the animation using a tag name
    let mut run_anim = Animation::start_from_tag(&anim_buffer, "Walk".to_string(), true).unwrap();

    let mut time = SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0x00FFFFFF;
        }

        // Update the animations to go to the correct frame
        walk_anim
            .update(&anim_buffer, time.elapsed().unwrap())
            .unwrap();
        jump_anim
            .update(&anim_buffer, time.elapsed().unwrap())
            .unwrap();
        run_anim
            .update(&anim_buffer, time.elapsed().unwrap())
            .unwrap();
        full_anim
            .update(&anim_buffer, time.elapsed().unwrap())
            .unwrap();

        // Render the frames
        anim_buffer
            .blit(&mut buffer, WIDTH, (4, 4), &walk_anim)
            .unwrap();
        anim_buffer
            .blit(&mut buffer, WIDTH, (36, 4), &jump_anim)
            .unwrap();
        anim_buffer
            .blit(&mut buffer, WIDTH, (68, 4), &run_anim)
            .unwrap();
        anim_buffer
            .blit(&mut buffer, WIDTH, (4, 68), &full_anim)
            .unwrap();

        // Draw all the frames separately
        for i in 0..11 {
            anim_buffer
                .blit_frame(&mut buffer, WIDTH, (32 * i + 4, 36), i as usize)
                .unwrap();
        }

        window.update_with_buffer(&buffer).unwrap();

        time = SystemTime::now();
    }
}

#[cfg(not(all(feature = "aseprite", feature = "image")))]
fn main() {
    // Ignore this example when not using image
}
