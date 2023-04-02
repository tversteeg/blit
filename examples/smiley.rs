#[cfg(feature = "image")]
fn main() {
    use blit::*;
    use image::GenericImageView;
    use minifb::*;

    const WIDTH: usize = 250;
    const HEIGHT: usize = 250;

    const MASK_COLOR: u32 = 0xFF_00_FF;

    let mut buffer: Vec<u32> = vec![0x00_FF_FF_FF; WIDTH * HEIGHT];

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

    let rgba = img.into_rgba8().to_blit_buffer_with_mask_color(MASK_COLOR);
    rgba.blit(&mut buffer, WIDTH, (img_size.0 as i32, 0));

    let img = image::open("examples/smiley_rgb.png").unwrap();
    println!("Loaded RGB image with size {:?}", img.dimensions());
    let img_size = img.dimensions();

    let rgb = img.into_rgba8().to_blit_buffer_with_alpha(127);
    rgb.blit(&mut buffer, WIDTH, (img_size.0 as i32, 0));

    let blit_size = rgb.size();
    let half_size = (blit_size.0 / 2, blit_size.1 / 2);

    // Draw the left half
    rgb.blit_rect(
        &mut buffer,
        WIDTH,
        (0, blit_size.1),
        (0, 0, half_size.0, blit_size.1),
    );

    // Draw the bottom right part
    rgb.blit_rect(
        &mut buffer,
        WIDTH,
        (half_size.0, (blit_size.1 + half_size.1)),
        (half_size.0, half_size.1, half_size.0, half_size.1),
    );

    let mut draw_countdown = 0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(mouse) = window.get_mouse_pos(MouseMode::Discard) {
            if draw_countdown <= 0 && window.get_mouse_down(MouseButton::Left) {
                let x_pos = mouse.0 as i32 - (img_size.0 / 2) as i32;
                let y_pos = mouse.1 as i32 - (img_size.1 / 2) as i32;
                rgb.blit(&mut buffer, WIDTH, (x_pos, y_pos));

                draw_countdown = 10;
            }
        }

        draw_countdown -= 1;

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}

#[cfg(not(feature = "image"))]
fn main() {
    // Ignore this example when not using image
}
