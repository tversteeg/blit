use aseprite::SpritesheetData;
use blit::{Animation, AnimationBlitBuffer, BlitBuffer, BlitExt};
use image::GenericImageView;
use softbuffer::GraphicsContext;

use web_time::SystemTime;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Color in the source image that needs to be masked to alpha
const MASK_COLOR: u32 = 0xFF_FF_FF;

// Backrgound color for the buffer
const BACKGROUND_COLOR: u32 = 0xFF_FF_CC_FF;

/// Load an aseprite generated spritesheet.
fn load_aseprite_image(img_bytes: &[u8], json: &str) -> (BlitBuffer, SpritesheetData) {
    // Open the animation spritesheet image
    let img = image::load_from_memory(img_bytes).unwrap();
    log::info!("Loaded image with size {:?}", img.dimensions());

    let rgba = img.into_rgb8();
    // Convert it to a blitbuffer
    let blit_buf = rgba.to_blit_buffer_with_mask_color(MASK_COLOR);

    // Open the spritesheet info
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

    // Create the animation buffer object
    let (anim_blit_buf, anim_info) = load_aseprite_image(
        include_bytes!("./king-by-buch.png"),
        include_str!("./king-by-buch.json"),
    );
    let anim_buffer = AnimationBlitBuffer::new(anim_blit_buf, anim_info);

    // Create the animations using hardcoded frames
    let mut walk_anim = Animation::start(0, 3, true);
    let mut jump_anim = Animation::start(4, 7, true);
    let mut full_anim = Animation::start(0, 11, true);

    // Create the animation using a tag name
    let mut run_anim = Animation::start_from_tag(&anim_buffer, "Walk".to_string(), true).unwrap();

    // Setup a winit window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    wasm::setup_canvas(&window);

    let mut graphics_context = unsafe { GraphicsContext::new(&window, &window) }.unwrap();

    // Keep track of how long each frame takes to render
    let mut time = SystemTime::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let width = window.inner_size().width as usize;
                let height = window.inner_size().height as usize;

                // Resize the buffer when needed
                if buffer.len() != width * height {
                    log::info!("Buffer resized to {width}x{height}, redrawing");

                    // Resize the buffer with empty colors
                    buffer.resize(width * height, BACKGROUND_COLOR);
                }

                // Clear the buffer
                buffer.fill(BACKGROUND_COLOR);

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
                    .blit(&mut buffer, width, (4, 4), &walk_anim)
                    .unwrap();
                anim_buffer
                    .blit(&mut buffer, width, (36, 4), &jump_anim)
                    .unwrap();
                anim_buffer
                    .blit(&mut buffer, width, (68, 4), &run_anim)
                    .unwrap();
                anim_buffer
                    .blit(&mut buffer, width, (4, 68), &full_anim)
                    .unwrap();

                // Draw all the frames separately
                for i in 0..11 {
                    anim_buffer
                        .blit_frame(&mut buffer, width, (32 * i + 4, 36), i as usize)
                        .unwrap();
                }

                graphics_context.set_buffer(&buffer, width as u16, height as u16);

                time = SystemTime::now();
            }
            Event::MainEventsCleared => {
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
