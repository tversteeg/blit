use blit::{aseprite::Slice9BlitBuffer, BlitBuffer, BlitExt};

use aseprite::SpritesheetData;
use image::GenericImageView;
use softbuffer::GraphicsContext;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Background color for the buffer
const BACKGROUND_COLOR: u32 = 0xFF_FF_CC_FF;

/// Load an aseprite generated spritesheet.
fn load_aseprite_image(img_bytes: &[u8], json: &str) -> (BlitBuffer, SpritesheetData) {
    // Open the animation spritesheet image
    let img = image::load_from_memory(img_bytes).unwrap();
    log::info!("Loaded image with size {:?}", img.dimensions());

    // Convert it to a blitbuffer
    let blit_buf = img.into_rgba8().to_blit_buffer_with_alpha(127);

    // Open the spritesheet info
    log::info!("{}", &json);
    let info: SpritesheetData = serde_json::from_str(json).unwrap();
    log::info!(
        "Loaded spritesheet JSON data with {} frames",
        info.frames.len()
    );

    (blit_buf, info)
}

fn main() {
    // The pixel buffer to fill
    let mut buffer: Vec<u32> = Vec::new();

    // Create the 9 slice buffer object
    let (blit_buf, info) = load_aseprite_image(
        include_bytes!("./button-9slice.png"),
        include_str!("./button-9slice.json"),
    );
    let slice9 = Slice9BlitBuffer::new(blit_buf, info);

    // Setup a winit window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    wasm::setup_canvas(&window);

    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    // Width of the canvas
    let mut width = 0;

    // Keep track of how long each frame takes to render
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                width = window.inner_size().width as usize;
                let height = window.inner_size().height as usize;

                // Resize the buffer when needed
                if buffer.len() != width * height {
                    log::info!("Buffer resized to {width}x{height}, redrawing");

                    // Clear the buffer before redrawing
                    buffer.fill(BACKGROUND_COLOR);

                    // Resize the buffer with empty colors
                    buffer.resize(width * height, BACKGROUND_COLOR);

                    // Blit the 9 slice pane with the size set by the cursor
                    slice9.blit(
                        &mut buffer,
                        width,
                        (10, 10, (width - 20) as i32, (height - 20) as i32),
                    );
                }

                graphics_context.set_buffer(&buffer, width as u16, height as u16);
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
                ..
            } if window_id == window.id() => {
                // Clear the buffer before redrawing
                buffer.fill(BACKGROUND_COLOR);

                // Blit the 9 slice pane with the size set by the cursor
                slice9.blit(
                    &mut buffer,
                    width,
                    (0, 0, position.x as i32, position.y as i32),
                );

                window.request_redraw();
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
        header.set_text_content(Some("Blit Example - Aseprite Animation"));
        body.append_child(&header).unwrap();
    }
}
