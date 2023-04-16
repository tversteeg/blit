use blit::{Blit, BlitBuffer, BlitOptions, ToBlitBuffer};

use image::GenericImageView;
use imgref::ImgVec;
use pixels::{wgpu::TextureFormat, PixelsBuilder, SurfaceTexture};

use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Window settings
const WIDTH: usize = 400;
const HEIGHT: usize = 300;

/// Color in the source image that needs to be masked to alpha.
const MASK_COLOR: u32 = 0xFF_00_FF;

// Size of a single character
const CHAR_WIDTH: i32 = 9;
const CHAR_HEIGHT: i32 = 10;

/// Draw the sprite completely.
fn frame1(dst: &mut [u32], buf: &ImgVec<u32>, font: &ImgVec<u32>) {
    let (center_x, center_y) = (WIDTH / 2 - buf.width() / 2, HEIGHT / 2 - buf.height() / 2);

    buf.blit_opt(
        dst,
        WIDTH,
        &BlitOptions::new((center_x as i32, center_y as i32)),
    );

    draw_text(dst, font, 0, "Single sprite without any clipping");
    draw_text(
        dst,
        font,
        HEIGHT as i32 - CHAR_WIDTH,
        "BlitOptions::new((center_x, center_y))",
    );
}

/// Draw the left half of the sprite using the area option.
fn frame2(dst: &mut [u32], buf: &ImgVec<u32>, font: &ImgVec<u32>) {
    let (src_width, src_height) = (buf.width() as i32 / 2, buf.height() as i32);
    let (center_x, center_y) = (
        WIDTH as i32 / 2 - src_width / 2,
        HEIGHT as i32 / 2 - src_height / 2,
    );

    buf.blit_opt(
        dst,
        WIDTH,
        &BlitOptions::new((center_x, center_y)).with_area((src_width, src_height)),
    );
    draw_text(
        dst,
        font,
        0,
        "Single sprite with the width area\ndivided by two",
    );
    draw_text(
        dst,
        font,
        HEIGHT as i32 - CHAR_HEIGHT * 2,
        "BlitOptions::new((center_x, center_y)\n\t.with_area((width / 2, height))",
    );
}

/// Draw the top half of the sprite two times by taking a sub rectangle from it.
fn frame3(dst: &mut [u32], buf: &ImgVec<u32>, font: &ImgVec<u32>) {
    let (src_width, src_height) = (buf.width() as i32, buf.height() as i32);

    buf.blit_opt(
        dst,
        WIDTH,
        &BlitOptions::new((5, 10)).with_sub_rect((0, 0, src_width, src_height / 2)),
    );
}

/// Draw an ASCII string.
fn draw_text(dst: &mut [u32], font: &ImgVec<u32>, mut y: i32, text: &str) {
    // First character in the image
    let char_start = '!';
    let char_end = '~';

    let mut x = 0;

    // Draw each character from the string
    text.chars().enumerate().for_each(|(i, ch)| {
        // Move the cursor
        x += CHAR_WIDTH;

        // Don't draw characters that are not in the picture
        if ch < char_start || ch > char_end {
            if ch == '\n' {
                x = 0;
                y += CHAR_HEIGHT;
            } else if ch == '\t' {
                x += CHAR_WIDTH * 3;
            }
            return;
        }

        // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
        let char_offset = (ch as u8 - char_start as u8) as i32 * CHAR_WIDTH;

        // Draw the character
        font.blit_opt(
            dst,
            WIDTH,
            &BlitOptions::new((x, y))
                .with_area((CHAR_WIDTH, CHAR_HEIGHT))
                .with_sub_rect((char_offset, 0, CHAR_WIDTH, CHAR_HEIGHT)),
        );
    });
}

/// Load the images and draw the window.
async fn run() {
    // Load an image with a mask color from disk
    let buf = image::load_from_memory(include_bytes!("./smiley_rgb.png"))
        .unwrap()
        .into_rgb8()
        .to_img_with_mask_color(MASK_COLOR);

    // Load the font image with mask color from disk
    let font = image::load_from_memory(include_bytes!("./ArtosSans.png"))
        .unwrap()
        .into_rgb8()
        .to_img_with_mask_color(MASK_COLOR);

    // Setup a winit window
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap();

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    wasm::setup_canvas(&window);

    let mut pixels = {
        let surface_texture = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, &window);
        PixelsBuilder::new(WIDTH as u32, HEIGHT as u32, surface_texture)
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
    let frames: Vec<fn(&mut [u32], &ImgVec<u32>, &ImgVec<u32>)> = vec![frame1, frame2, frame3];

    // Keep track of how long each frame takes to render
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // Redraw the pixel buffer
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Clear the buffer
                pixels.frame_mut().fill(0);

                // Convert the [u8] * 4 array of pixels to [u32] and draw the frame
                frames[current_frame](bytemuck::cast_slice_mut(pixels.frame_mut()), &buf, &font);

                if let Err(err) = pixels.render() {
                    log::error!("Pixels error:\n{err}");
                }
            }

            // Go to the next frame when the mouse is down
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state: ElementState::Released,
                        ..
                    },
                window_id,
                ..
            } if window_id == window.id() => {
                current_frame += 1;
                // Wrap around
                if current_frame >= frames.len() {
                    current_frame = 0;
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
        wasm_bindgen_futures::spawn_local(run());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(run());
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use winit::{platform::web::WindowExtWebSys, window::Window};

    /// Run main on the browser.
    #[wasm_bindgen(start)]
    pub fn run() {
        console_log::init_with_level(log::Level::Info).expect("error initializing logger");

        super::main();
    }

    /// Attach the winit window to a canvas.
    pub fn setup_canvas(window: &Window) {
        log::debug!("Binding window to HTML canvas");

        let canvas = window.canvas();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();
        body.style().set_css_text("text-align: center");

        body.append_child(&canvas).unwrap();
        canvas.style().set_css_text("display:block; margin: auto");
        canvas.set_width(600);
        canvas.set_height(400);

        let header = document.create_element("h2").unwrap();
        header.set_text_content(Some("Blit Example - Aseprite Animation"));
        body.append_child(&header).unwrap();
    }
}
