use blit::{Blit, BlitBuffer, ToBlitBuffer};
use image::GenericImageView;
use softbuffer::GraphicsContext;
use winit::{
    event::{Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

/// Color in the source image that needs to be masked to alpha.
const MASK_COLOR: u32 = 0xFF_00_FF;

/// Background color for the buffer.
const BACKGROUND_COLOR: u32 = 0xFF_FF_CC_FF;

/// Redraw the static images on the buffer.
fn redraw(buffer: &mut [u32], rgb: &BlitBuffer, rgba: &BlitBuffer, width: usize) {
    log::info!("Redrawing full buffer");

    // Draw the images at full size
    rgba.blit(buffer, width, (0, 0));

    let half_size = (rgb.width() / 2, rgb.height() / 2);

    // Draw the left half
    rgb.blit_subrect(
        buffer,
        width,
        (0, rgb.height()),
        (0, 0, half_size.0, rgb.height()),
    );

    // Draw the bottom right part
    rgb.blit_subrect(
        buffer,
        width,
        (half_size.0, (rgb.height() + half_size.1)),
        (half_size.0, half_size.1, half_size.0, half_size.1),
    );

    // Draw it tiling
    rgb.blit_area(
        buffer,
        width,
        (
            0,
            rgb.height() * 2,
            rgb.width() + half_size.0,
            rgb.height() * 2 + half_size.1,
        ),
    );

    // Draw a third part of it tiling
    rgb.blit_area_subrect(
        buffer,
        width,
        (
            rgb.width() * 2,
            0,
            rgb.width() + half_size.0,
            rgb.height() * 2 + half_size.1,
        ),
        (
            rgb.width() / 3,
            rgb.height() / 3,
            rgb.width() / 3,
            rgb.height() / 3,
        ),
    );
}

fn main() {
    // The pixel buffer to fill
    let mut buffer: Vec<u32> = Vec::new();

    // Load the image from disk
    let img = image::load_from_memory(include_bytes!("./smiley_rgba.png")).unwrap();
    log::info!("Loaded RGBA image with size {:?}", img.dimensions());

    // Convert the image to a blit buffer
    let rgba = img.into_rgba8().to_blit_buffer_with_alpha(127);

    // Load another image with a mask color from disk
    let img = image::load_from_memory(include_bytes!("./smiley_rgb.png")).unwrap();
    log::info!("Loaded RGB image with size {:?}", img.dimensions());

    // Also convert it to a blit buffer
    let rgb = img.into_rgb8().to_blit_buffer_with_mask_color(MASK_COLOR);

    // Setup a winit window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    wasm::setup_canvas(&window);

    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    // Keep track of the mouse position
    let mut mouse_pos = (0, 0);
    let mut width = 0;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                width = window.inner_size().width as usize;
                let height = window.inner_size().height as usize;

                // Redraw the whole buffer if it resized
                if buffer.len() != width * height {
                    log::info!("Buffer resized to {width}x{height}, redrawing");

                    // Clear the buffer first
                    buffer.fill(BACKGROUND_COLOR);

                    // Resize the buffer with empty values
                    buffer.resize(width * height, BACKGROUND_COLOR);

                    // Redraw all static sprites
                    redraw(&mut buffer, &rgb, &rgba, width);
                }

                graphics_context.set_buffer(&buffer, width as u16, height as u16);
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        ..
                    },
                window_id,
                ..
            } if window_id == window.id() => {
                // Draw the picture at the center of the cursor
                let x_pos = mouse_pos.0 - (rgb.width() / 2) as i32;
                let y_pos = mouse_pos.1 - (rgb.height() / 2) as i32;
                rgb.blit(&mut buffer, width, (x_pos, y_pos));

                // Tell the window to redraw another frame
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
                ..
            } if window_id == window.id() => {
                mouse_pos.0 = position.x as i32;
                mouse_pos.1 = position.y as i32;
            }
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

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;
    use winit::{platform::web::WindowExtWebSys, window::Window};

    /// Run main on the browser.
    #[wasm_bindgen(start)]
    pub fn run() {
        console_log::init_with_level(log::Level::Debug).expect("error initializing logger");

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
        header.set_text_content(Some("Blit Example - Smiley"));
        body.append_child(&header).unwrap();
    }
}
