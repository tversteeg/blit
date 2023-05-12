use blit::{
    geom::Rect,
    geom::{Coordinate, Size},
    slice::Slice,
    Blit, BlitBuffer, ToBlitBuffer,
};

use num_traits::ToPrimitive;
use pixels::{PixelsBuilder, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
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
fn frame_intro(
    dst: &mut [u32],
    _buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    _mouse: Coordinate,
) {
    draw_text(dst, font, 0, "This is an interactive showcase of the\n'blit' crate, you can interact with the\nrendering by moving the cursor\n\nGo to the next showcase item by clicking\nthe left mouse button\n\nGo to the previous showcase item by\nclicking the right mouse button");
}

/// Draw the sprite completely.
fn frame_complete(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Coordinate,
) {
    let center = (DST_SIZE / 2 - buf.size() / 2).as_tuple();

    buf.blit(dst, DST_SIZE).position(center).draw();
    buf.blit(dst, DST_SIZE)
        .position(mouse - buf.size() / 2)
        .draw();

    draw_text(dst, font, 0, "Blit the full sprite");
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 2,
        "buf.blit(dst, dst_size)\n\t.position(mouse).draw()",
    );
}

/// Draw the left half of the sprite using the area option.
fn frame_area(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Coordinate,
) {
    let center = (DST_SIZE / 2 - buf.size() / 2).as_coordinate();
    let mut sprite_size = buf.size();
    sprite_size.width = (mouse.x - center.x as i32).clamp(0, buf.width() as i32) as u32;

    buf.blit(dst, DST_SIZE)
        .position(center)
        .area(sprite_size)
        .draw();

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
        "buf.blit(dst, dst_size).area(mouse).draw()",
    );
}

/// Draw the top half of the sprite by taking a sub rectangle from it.
fn frame_sub_rect(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Coordinate,
) {
    let center = (DST_SIZE / 2 - buf.size() / 2).as_coordinate();
    let mut sprite_size = buf.size();
    sprite_size.height = (mouse.y - center.y).clamp(0, buf.height() as i32) as u32;

    buf.blit(dst, DST_SIZE)
        .position(center)
        .sub_rect(Rect::new(0, 0, sprite_size))
        .draw();

    draw_text(
        dst,
        font,
        0,
        "Alternatively only show a part of the image\nby using a sub-rectangle",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 3,
        "buf.blit(dst, dst_size)\n\t.sub_rect((0, 0, width, mouse_y)).\n\tdraw()",
    );
}

/// Draw the sprite with shifed UV coordinates.
fn frame_uv(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: Coordinate,
) {
    let center = (DST_SIZE / 2 - buf.size() / 2).as_coordinate();

    buf.blit(dst, DST_SIZE).position(center).uv(mouse).draw();

    draw_text(
        dst,
        font,
        0,
        "Shifting the UV coordinates allows you to\nscroll the image around",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 3,
        "buf.blit(dst, dst_size)\n\t.uv(mouse)\n\t.draw()",
    );
}

/*
/// Draw the middle section of the sprite by taking a sub rectangle from it.
fn frame_sub_rect2(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let src_size = buf.size() / 2;
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y).with_sub_rect(SubRect::new(
            (mouse.0 - center_x as i32).clamp(0, src_size.width as i32),
            (mouse.1 - center_y as i32).clamp(0, src_size.height as i32),
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
fn frame_area2(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (40, 40);

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y).with_area(Size::new(
            (mouse.0 - offset_x).max(1),
            (mouse.1 - offset_y).max(1),
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
fn frame_tiled(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (40, 40);

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_area(Size::new(
                (mouse.0 - offset_x).max(1) as u32,
                (mouse.1 - offset_y).max(1) as u32,
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
fn frame_slice9(
    dst: &mut [u32],
    _buf: &BlitBuffer,
    scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (60, 60);

    scalable_buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_slice9((11, 9, 6, 4))
            .with_area(Size::new(
                (mouse.0 - offset_x).max(1),
                (mouse.1 - offset_y).max(1),
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
fn frame_vertical_slice(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (60, 60);

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_vertical_slice(Slice::binary_first(buf.width() / 2))
            .with_area(Size::new((mouse.0 - offset_x).max(1), buf.height())),
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

/// Draw the sprite completely with a mask.
fn frame_mask(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();

    buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(center_x, center_y).with_mask((mouse.0, mouse.1, 200, 200)),
    );

    draw_text(
        dst,
        font,
        0,
        "Blit the full sprite with a mask on the\ntarget buffer",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 2,
        "BlitOptions::new(mouse)\n\t.with_mask((mouse_x, mouse_y, 10, 10))",
    );
}
*/

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
        font.blit(dst, DST_SIZE)
            .position((x, y))
            .sub_rect(Rect::new(char_offset, 0, CHAR_SIZE))
            .draw();
    });
}

/// Load the images and draw the window.
async fn run() {
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
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(
        DST_SIZE.width as f64 * 2.0 + 10.0,
        DST_SIZE.height as f64 * 2.0 + 10.0,
    );
    let mut window_builder = WindowBuilder::new()
        .with_title("Blit Showcase")
        .with_inner_size(size);

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowBuilderExtWebSys;

        window_builder = window_builder.with_canvas(Some(wasm::setup_canvas()));
    }

    let window = window_builder.build(&event_loop).unwrap();

    let mut pixels = {
        let surface_texture =
            SurfaceTexture::new(DST_SIZE.width * 2 + 10, DST_SIZE.height * 2 + 10, &window);
        PixelsBuilder::new(DST_SIZE.width, DST_SIZE.height, surface_texture)
            .clear_color(pixels::wgpu::Color {
                r: 0.3,
                g: 0.1,
                b: 0.3,
                a: 1.0,
            })
            .build_async()
            .await
    }
    .unwrap();

    // Cursor position
    let mut mouse = Coordinate::new(0, 0);

    // Which frame to draw
    let mut current_frame = 0;

    // All frame drawing functions, cycled by clicking
    let frames: Vec<fn(&mut [u32], &BlitBuffer, &BlitBuffer, &BlitBuffer, Coordinate)> = vec![
        frame_intro,
        frame_complete,
        frame_area,
        frame_sub_rect,
        frame_uv,
        /*
        frame_sub_rect2,
        frame_area2,
        frame_tiled,
        frame_slice9,
        frame_vertical_slice,
        frame_mask,
        */
    ];

    // Keep track of how long each frame takes to render
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // Redraw the pixel buffer
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Clear the buffer
                pixels.frame_mut().fill(0);

                // Convert the [u8] * 4 array of pixels to [u32] and draw the frame
                frames[current_frame](
                    bytemuck::cast_slice_mut(pixels.frame_mut()),
                    &buf,
                    &scalable_buf,
                    &font,
                    mouse,
                );

                // Blit draws the pixels in RGBA format, but the pixels crate expects BGRA, so convert it
                pixels.frame_mut().chunks_exact_mut(4).for_each(|color| {
                    let (r, g, b, a) = (color[0], color[1], color[2], color[3]);

                    color[0] = b;
                    color[1] = g;
                    color[2] = r;
                    color[3] = a;
                });

                if let Err(err) = pixels.render() {
                    log::error!("Pixels error:\n{err}");
                }
            }

            // Go to the next frame when the mouse is down
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button,
                        state: ElementState::Released,
                        ..
                    },
                window_id,
                ..
            } if window_id == window.id() => {
                if button == MouseButton::Left {
                    current_frame += 1;

                    // Wrap around
                    if current_frame >= frames.len() {
                        current_frame = 0;
                    }
                } else if button == MouseButton::Right {
                    // Wrap around
                    if current_frame == 0 {
                        current_frame = frames.len();
                    }

                    current_frame -= 1;
                }

                // Tell the window to redraw another frame
                window.request_redraw();
            }
            // Handle the mouse cursor movement
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
                ..
            } if window_id == window.id() => {
                // Update the mouse position
                let mouse_pos = pixels
                    .window_pos_to_pixel((position.x as f32, position.y as f32))
                    .unwrap_or_default();
                mouse.x = mouse_pos.0 as i32;
                mouse.y = mouse_pos.1 as i32;

                // Draw another frame
                window.request_redraw();
            }
            // Resize the window
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == window.id() => {
                pixels.resize_surface(size.width, size.height).unwrap();

                // Draw another frame
                window.request_redraw();
            }
            // Close the window
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(run());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    /// Attach the winit window to a canvas.
    pub fn setup_canvas() -> HtmlCanvasElement {
        log::debug!("Binding window to HTML canvas");

        let window = web_sys::window().unwrap();

        let document = window.document().unwrap();
        let body = document.body().unwrap();
        body.style().set_css_text("text-align: center");

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_id("canvas");
        body.append_child(&canvas).unwrap();
        canvas.style().set_css_text("display:block; margin: auto");

        let header = document.create_element("h2").unwrap();
        header.set_text_content(Some("Blit Showcase"));
        body.append_child(&header).unwrap();

        canvas
    }
}
