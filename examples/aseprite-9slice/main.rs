use blit::{aseprite::Slice9BlitBuffer, Blit, BlitBuffer, ToBlitBuffer};

use aseprite::SpritesheetData;
use image::GenericImageView;
use pixels::{wgpu::TextureFormat, PixelsBuilder, SurfaceTexture};

use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Window settings
const WIDTH: usize = 300;
const HEIGHT: usize = 200;

/// Load an aseprite generated spritesheet.
fn load_aseprite_image(img_bytes: &[u8], json: &str) -> (BlitBuffer, SpritesheetData) {
    // Open the animation spritesheet image
    let img = image::load_from_memory(img_bytes).unwrap();
    log::info!("Loaded image with size {:?}", img.dimensions());

    // Convert it to a blitbuffer
    let blit_buf = img.into_rgba8().to_blit_buffer_with_alpha(127);

    // Open the spritesheet info
    let info: SpritesheetData = serde_json::from_str(json).unwrap();
    log::info!(
        "Loaded spritesheet JSON data with {} frames",
        info.frames.len()
    );

    (blit_buf, info)
}

async fn run() {
    // Create the 9 slice buffer object
    let (blit_buf, info) = load_aseprite_image(
        include_bytes!("./button-9slice.png"),
        include_str!("./button-9slice.json"),
    );
    let slice9 = Slice9BlitBuffer::new(blit_buf, info).unwrap();

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
                r: 1.0,
                g: 0.8,
                b: 1.0,
                a: 1.0,
            })
            .texture_format(TextureFormat::Bgra8UnormSrgb)
            .build_async()
            .await
    }
    .unwrap();

    // Cursor position
    let mut mouse = (0, 0);

    // Keep track of how long each frame takes to render
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            // Redraw the pixel buffer
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                // Clear the buffer
                pixels.frame_mut().fill(0);

                // Blit the 9 slice pane with the size set by the cursor
                slice9.blit_area(
                    bytemuck::cast_slice_mut(pixels.frame_mut()),
                    WIDTH,
                    (5, 5, mouse.0 - 5, mouse.1 - 5),
                );

                // Blit a sub rect of the 9 slice pane with the size set by the cursor
                slice9.blit_area_subrect(
                    bytemuck::cast_slice_mut(pixels.frame_mut()),
                    WIDTH,
                    (
                        mouse.0,
                        mouse.1,
                        WIDTH as i32 - mouse.0 - 5,
                        HEIGHT as i32 - mouse.1 - 5,
                    ),
                    (3, 3, slice9.size().0 - 6, slice9.size().1 - 6),
                );

                if let Err(err) = pixels.render() {
                    log::error!("Pixels error:\n{err}");
                }
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
