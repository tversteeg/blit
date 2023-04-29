use std::rc::Rc;

use blit::{Blit, BlitBuffer, BlitOptions, Size, SubRect, ToBlitBuffer};

use num_traits::ToPrimitive;
use pixels::{wgpu::TextureFormat, PixelsBuilder, SurfaceTexture};
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
fn frame0(
    dst: &mut [u32],
    _buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    _mouse: (i32, i32),
) {
    draw_text(dst, font, 0, "Go to the next showcase item by clicking\nthe left mouse button.\n\nGo to the previous showcase item by\nclicking the right mouse button.");
}

/// Draw the sprite completely.
fn frame1(
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
        &BlitOptions::new_position(center_x, center_y),
    );
    buf.blit(dst, DST_SIZE, &BlitOptions::new_position_tuple(mouse));

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
    mouse: (i32, i32),
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();
    let mut sprite_size = buf.size();
    sprite_size.width = (mouse.0 - center_x as i32).clamp(0, buf.width() as i32) as u32;

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
    mouse: (i32, i32),
) {
    let (center_x, center_y) = (DST_SIZE / 2 - buf.size() / 2).as_tuple();
    let mut sprite_size = buf.size();
    sprite_size.height = (mouse.1 - center_y as i32).clamp(0, buf.height() as i32) as u32;

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
fn frame5(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (20, 20);

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
fn frame6(
    dst: &mut [u32],
    buf: &BlitBuffer,
    _scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (20, 20);

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
fn frame7(
    dst: &mut [u32],
    _buf: &BlitBuffer,
    scalable_buf: &BlitBuffer,
    font: &BlitBuffer,
    mouse: (i32, i32),
) {
    let (offset_x, offset_y) = (20, 20);

    scalable_buf.blit(
        dst,
        DST_SIZE,
        &BlitOptions::new_position(offset_x, offset_y)
            .with_slice9(10, 20, 10, 20)
            .with_area(Size::new(
                (mouse.0 - offset_x).max(1),
                (mouse.1 - offset_y).max(1),
            )),
    );

    draw_text(
        dst,
        font,
        0,
        "Which part of the sprite gets scaled can be\ncontrolled with slices",
    );
    draw_text(
        dst,
        font,
        DST_SIZE.height - CHAR_SIZE.height * 3,
        "BlitOptions::new_position(position)\n\t.with_slice9(11, 17, 9, 13)\n\t.with_area(mouse)",
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
async fn run() {
    // Load an image with a mask color from disk
    let buf = image::load_from_memory(include_bytes!("./smiley_rgb.png"))
        .unwrap()
        .into_rgb8()
        .to_blit_buffer_with_mask_color(MASK_COLOR)
        .unwrap();

    // Load the font image with mask color from disk
    let font = image::load_from_memory(include_bytes!("./ArtosSans.png"))
        .unwrap()
        .into_rgb8()
        .to_blit_buffer_with_mask_color(MASK_COLOR)
        .unwrap();

    // Load a scalable image with a mask color from disk
    let scalable_buf = image::load_from_memory(include_bytes!("./9slice.png"))
        .unwrap()
        .into_rgba8()
        .to_blit_buffer_with_alpha(127)
        .unwrap();

    // Setup a winit window
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(DST_SIZE.width as f64, DST_SIZE.height as f64);
    let window = Rc::new(
        WindowBuilder::new()
            .with_title("Blit Showcase")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap(),
    );

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    wasm::setup_canvas(window.clone());

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(DST_SIZE.width, DST_SIZE.height, window.as_ref());
        PixelsBuilder::new(DST_SIZE.width, DST_SIZE.height, surface_texture)
            .clear_color(pixels::wgpu::Color {
                r: 0.3,
                g: 0.1,
                b: 0.3,
                a: 1.0,
            })
            .texture_format(TextureFormat::Bgra8UnormSrgb)
            .build_async()
            .await
    }
    .unwrap();

    // Cursor position
    let mut mouse = (0, 0);

    // Which frame to draw
    let mut current_frame = 0;

    // All frame drawing functions, cycled by clicking
    let frames: Vec<fn(&mut [u32], &BlitBuffer, &BlitBuffer, &BlitBuffer, (i32, i32))> = vec![
        frame0, frame1, frame2, frame3, frame4, frame5, frame6, frame7,
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
                mouse.0 = mouse_pos.0 as i32;
                mouse.1 = mouse_pos.1 as i32;

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
    use std::rc::Rc;
    use wasm_bindgen::{closure::Closure, JsCast};
    use web_sys::{Element, Event};
    use winit::{dpi::LogicalSize, platform::web::WindowExtWebSys, window::Window};

    /// Attach the winit window to a canvas.
    pub fn setup_canvas(window: Rc<Window>) {
        log::debug!("Binding window to HTML canvas");

        // Retrieve current width and height dimensions of browser client window
        let get_window_size = || {
            let client_window = web_sys::window().unwrap();
            LogicalSize::new(
                client_window.inner_width().unwrap().as_f64().unwrap(),
                client_window.inner_height().unwrap().as_f64().unwrap(),
            )
        };

        let window = Rc::clone(&window);

        // Initialize winit window with current dimensions of browser client
        window.set_inner_size(get_window_size());

        let client_window = web_sys::window().unwrap();

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(&Element::from(window.canvas())).ok())
            .expect("couldn't append canvas to document body");

        // Listen for resize event on browser client. Adjust winit window dimensions
        // on event trigger
        let closure = Closure::wrap(Box::new(move |_e: Event| {
            let size = get_window_size();
            window.set_inner_size(size)
        }) as Box<dyn FnMut(_)>);
        client_window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}
