use blit::{geom::Size, geom::SubRect, slice::Slice, Blit, BlitBuffer, BlitOptions, ToBlitBuffer};

use num_traits::ToPrimitive;
use pixel_game_lib::{
    vek::{Extent2, Vec2},
    window::{Key, WindowConfig},
};

// Window settings
const DST_SIZE: Size = Size {
    width: 400,
    height: 300,
};

/// Color in the source image that needs to be masked to alpha.
const MASK_COLOR: u32 = 0xFF_00_FF;

/// Size of a single character.
const CHAR_SIZE: Size = Size {
    width: 9,
    height: 10,
};

/// Show the text for clicking.
fn frame0(
    dst: &mut [u32],
    _buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    _mouse: Vec2<i32>,
) {
    draw_text(dst, font, 0, "This is an interactive showcase of the\n'blit' crate, you can interact with the\nrendering by moving the cursor\n\nGo to the next showcase item by clicking\nthe left mouse button\n\nGo to the previous showcase item by\nclicking the right mouse button");
}

/// Draw the sprite completely.
fn frame1(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y),
    );
    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position_tuple(mouse.into_tuple()),
    );

    draw_text(dst, font, 0, "Blit the full sprite");
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height,
        "BlitOptions::new(mouse)",
    );
}

/// Draw the left half of the sprite using the area option.
fn frame2(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();
    let mut sprite_size = buf.size();
    sprite_size.width = (mouse.x - center_x as i32).clamp(0, buf.width() as i32) as u32;

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y)
            .with_sub_rect(SubRect::from_size(sprite_size)),
    );

    draw_text(
        dst,
        font,
        0,
        "Only show a part of the image by using\na smaller area",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 2,
        "BlitOptions::new(position)\n\t.with_area((mouse_x, height))",
    );
}

/// Draw the top half of the sprite by taking a sub rectangle from it.
fn frame3(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();
    let mut sprite_size = buf.size();
    sprite_size.height = (mouse.y - center_y as i32).clamp(0, buf.height() as i32) as u32;

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y)
            .with_sub_rect(SubRect::from_size(sprite_size)),
    );

    draw_text(
        dst,
        font,
        0,
        "Alternatively only show a part of the image\nby using a sub-rectangle",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 2,
        "BlitOptions::new(position)\n\t.with_sub_rect((0, 0, width, mouse_y))",
    );
}

/// Draw the middle section of the sprite by taking a sub rectangle from it.
fn frame4(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let src_size = buf.size() / 2;
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y).with_sub_rect(SubRect::new(
            (mouse.x - center_x as i32).clamp(0, src_size.width as i32),
            (mouse.y - center_y as i32).clamp(0, src_size.height as i32),
            src_size,
        )),
    );

    draw_text(
        dst,
        font,
        0,
        "Sub-rectangles can be used to draw any\npart of the sprite",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 4,
        "BlitOptions::new(position)\n\t.with_sub_rect(\n\t\t(mouse_x, mouse_y, w/2, h/2)\n\t)",
    );
}

/// Draw the full sprite tiled multiple times for each dimension.
fn frame5(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (offset_x, offset_y) = (40, 40);

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y).with_area(Size::new(
            (mouse.x - offset_x).max(1),
            (mouse.y - offset_y).max(1),
        )),
    );

    draw_text(
        dst,
        font,
        0,
        "Areas bigger than the sprite can be used\nfor tiling",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 2,
        "BlitOptions::new(position)\n\t.with_area(mouse)",
    );
}

/// Draw a sub-rectangle of the sprite tiled multiple times for each dimension.
fn frame6(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (offset_x, offset_y) = (40, 40);

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_area(Size::new(
                (mouse.x - offset_x).max(1) as u32,
                (mouse.y - offset_y).max(1) as u32,
            ))
            .with_sub_rect(SubRect::new(0, 70, Size::new(34, 32))),
    );

    draw_text(
        dst,
        font,
        0,
        "Combining areas with sub-rectangles allows\nfor creating custom tiling patterns",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 3,
        "BlitOptions::new(position)\n\t.with_area(mouse)\n\t.with_sub_rect((0, 70, 34, 32))",
    );
}

/// Draw the full sprite as slice9 scaling.
fn frame7(
    dst: &mut [u32],
    _buf: &BlitBuffer,
    scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (offset_x, offset_y) = (60, 60);

    scalable_buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_slice9((11, 9, 6, 4))
            .with_area(Size::new(
                (mouse.x - offset_x).max(1),
                (mouse.y - offset_y).max(1),
            )),
    );

    draw_text(
        dst,
        font,
        0,
        "Which part of the sprite gets scaled can be\ncontrolled with slices, the most common and\nuseful is the 9 slice where the middle part\ngets scaled",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 3,
        "BlitOptions::new_position(position)\n\t.with_slice9((10, 10, 10, 10))\n\t.with_area(mouse)",
    );
}

/// Draw the full sprite split in half scaling the left side.
fn frame8(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (offset_x, offset_y) = (60, 60);

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_vertical_slice(Slice::binary_first(buf.width() / 2))
            .with_area(Size::new((mouse.x - offset_x).max(1), buf.height())),
    );

    draw_text(
        dst,
        font,
        0,
        "More fine-grained control for slices is\nalso possible",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 5,
        "BlitOptions::new_position(position)\n\t.with_horizontal_slice(\n\t\tSlice::binary_first(w/2)\n\t)\n\t.with_area((mouse_x, h))",
    );
}

/// Draw the full sprite with a mask.
fn frame9(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Vec2<i32>,
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y).with_mask((
            mouse.x - 30,
            mouse.y - 30,
            60,
            60,
        )),
    );

    draw_text(
        dst,
        font,
        0,
        "Clipping based on the destination view\ncan also be achieved with a mask",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 5,
        "BlitOptions::new_position(position)\n\t.with_mask((mouse_x, mouse_y, 30, 30))",
    );
}

/// Draw an ASCII string.
fn draw_text(dst: &mut [u32], font: &BlitBuffer, y: impl ToPrimitive, text: &str) {
    // First character in the image
    let char_start = '!';
    let char_end = '~';

    let mut x = 0;
    let mut y = y.to_i32().unwrap_or_default();

    // Draw each character from the string
    text.chars().for_each(|ch| {
        // Move the cursor
        x += CHAR_SIZE.width as i32;

        // Don't draw characters that are not in the picture
        if ch < char_start || ch > char_end {
            if ch == '\n' {
                x = 0;
                y += CHAR_SIZE.height as i32;
            } else if ch == '\t' {
                x += CHAR_SIZE.width as i32 * 3;
            }
            return;
        }

        // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
        let char_offset = (ch as u8 - char_start as u8) as u32 * CHAR_SIZE.width;

        // Draw the character
        font.blit(
            dst,
            DST_SIZE,
            &BlitOptions::new_position(x, y).with_sub_rect(SubRect::new(char_offset, 0, CHAR_SIZE)),
        );
    });
}

/// Load the images and draw the window.
fn main() {
    // Load an image with a mask color from disk
    let buf = image::load_from_memory(include_bytes!("./smiley_rgb.png"))
        .unwrap()
        .into_rgb8()
        .to_blit_buffer_with_mask_color(MASK_COLOR);

    // Load the font image with mask color from disk
    let font = image::load_from_memory(include_bytes!("./ArtosSans.png"))
        .unwrap()
        .into_rgb8()
        .to_blit_buffer_with_mask_color(MASK_COLOR);

    // Load a scalable image with a mask color from disk
    let scalable_buf = image::load_from_memory(include_bytes!("./9slice.png"))
        .unwrap()
        .into_rgba8()
        .to_blit_buffer_with_alpha(127);

    // Setup a winit window
    let window_config = WindowConfig {
        buffer_size: Extent2::new(DST_SIZE.width, DST_SIZE.height).as_(),
        scaling: 2,
        title: "Blit Showcase".to_string(),
        ..Default::default()
    };

    // All frame drawing functions, cycled by clicking
    type Frames = Vec<fn(&mut [u32], &BlitBuffer, &BlitBuffer, &BlitBuffer, Vec2<i32>)>;
    let frames: Frames = vec![
        frame0, frame1, frame2, frame3, frame4, frame5, frame6, frame7, frame8, frame9,
    ];

    // The game state container everything specified above
    struct State {
        buf: BlitBuffer,
        font: BlitBuffer,
        scalable_buf: BlitBuffer,
        // Current mouse position
        mouse: Vec2<usize>,
        // Which frame to draw
        current_frame: usize,
        frames: Frames,
    }

    // Keep track of how long each frame takes to render
    pixel_game_lib::window(
        State {
            buf,
            font,
            scalable_buf,
            frames,
            mouse: Vec2::zero(),
            current_frame: 0,
        },
        window_config.clone(),
        move |state, input, mouse, _dt| {
            // Move to the next "slide" when clicking
            if input.mouse_released(0) {
                state.current_frame += 1;

                // Wrap around
                if state.current_frame >= state.frames.len() {
                    state.current_frame = 0;
                }
            } else if input.mouse_released(1) {
                // Wrap around
                if state.current_frame == 0 {
                    state.current_frame = state.frames.len();
                }

                state.current_frame -= 1;
            }

            // Update the mouse coordinates
            if let Some(mouse) = mouse {
                state.mouse = mouse;
            }

            // Exit when escape is pressed
            input.key_pressed(Key::Escape)
        },
        |state, canvas, _dt| {
            // Clear the buffer
            canvas.fill(0xFF955995);

            // Convert the [u8] * 4 array of pixels to [u32] and draw the frame
            state.frames[state.current_frame](
                canvas.raw_buffer(),
                &state.buf,
                &state.scalable_buf,
                &state.font,
                state.mouse.as_(),
            );
        },
    )
    .expect("Error opening window");
}
