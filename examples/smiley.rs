extern crate blit;
extern crate image;
extern crate minifb;

use blit::*;
use image::GenericImageView;
use minifb::*;

const WIDTH: usize = 250;
const HEIGHT: usize = 250;

const MASK_COLOR: u32 = 0xFF00FF;

#[cfg(feature = "image")]
fn main() {
    let mut buffer: Vec<u32> = vec![0x00FFFFFF; WIDTH * HEIGHT];

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new(
        "Blit Example - ESC to exit & click to draw",
        WIDTH,
        HEIGHT,
        options,
    )
    .expect("Unable to open window");

    let img = image::open("examples/smiley_rgba.png").unwrap();
    println!("Loaded RGBA image with size {:?}", img.dimensions());
    let img_size = img.dimensions();

    let rgb = img.as_rgba8().unwrap();
    rgb.blit(
        &mut buffer,
        WIDTH,
        (img_size.0 as i32, 0),
        Color::from_u32(MASK_COLOR),
    );

    let img = image::open("examples/smiley_rgb.png").unwrap();
    println!("Loaded RGB image with size {:?}", img.dimensions());
    let img_size = img.dimensions();

    let rgb = img.as_rgb8().unwrap();
    rgb.blit(&mut buffer, WIDTH, (0, 0), Color::from_u32(0xFFFFFF));

    // Convert the image to a specific blit buffer type which is a lot faster
    let blit_buf = rgb.to_blit_buffer(Color::from_u32(MASK_COLOR));

    // It's not necessarily to use the `as_rgb*` for this
    let blit_buf = blit_buffer(&img, Color::from_u32(MASK_COLOR));

    // Save the buffer to disk and load it again
    blit_buf.save("smiley.blit").unwrap();
    let blit_buf = BlitBuffer::open("smiley.blit").unwrap();

    let blit_size = blit_buf.size();
    let half_size = (blit_size.0 / 2, blit_size.1 / 2);

    // Draw the left half
    blit_buf.blit_rect(
        &mut buffer,
        WIDTH,
        (0, blit_size.1),
        (0, 0, half_size.0, blit_size.1),
    );

    // Draw the bottom right part
    blit_buf.blit_rect(
        &mut buffer,
        WIDTH,
        (half_size.0, (blit_size.1 + half_size.1)),
        (half_size.0, half_size.1, half_size.0, half_size.1),
    );

    let mut draw_countdown = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            if draw_countdown <= 0 && window.get_mouse_down(MouseButton::Left) {
                let x_pos = mouse.0 as i32 - (img_size.0 / 2) as i32;
                let y_pos = mouse.1 as i32 - (img_size.1 / 2) as i32;
                blit_buf.blit(&mut buffer, WIDTH, (x_pos, y_pos));

                draw_countdown = 10;
            }
        });

        draw_countdown -= 1;

        window.update_with_buffer(&buffer).unwrap();
    }
}

#[cfg(not(feature = "image"))]
fn main() {
    // Ignore this example when not using image
}
