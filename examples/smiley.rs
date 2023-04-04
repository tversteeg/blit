use blit::*;
use image::GenericImageView;
use softbuffer::GraphicsContext;
use winit::{
    event::{Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// Color in the source image that needs to be masked to alpha
const MASK_COLOR: u32 = 0xFF_00_FF;

// Backrgound color for the buffer
const BACKGROUND_COLOR: u32 = 0xFF_FF_CC_FF;

/// Redraw the static images on the buffer.
fn redraw(buffer: &mut [u32], rgb: &BlitBuffer, rgba: &BlitBuffer, width: usize) {
    // Draw the images at full size
    rgba.blit(buffer, width, (0, 0));
    rgb.blit(buffer, width, (rgb.width(), 0));

    let half_size = (rgb.width() / 2, rgb.height() / 2);

    // Draw the left half
    rgb.blit_rect(
        buffer,
        width,
        (0, rgb.height()),
        (0, 0, half_size.0, rgb.height()),
    );

    // Draw the bottom right part
    rgb.blit_rect(
        buffer,
        width,
        (half_size.0, (rgb.height() + half_size.1)),
        (half_size.0, half_size.1, half_size.0, half_size.1),
    );
}

fn main() {
    // The pixel buffer to fill
    let mut buffer: Vec<u32> = Vec::new();

    // Load the image from disk
    let img = image::open("examples/smiley_rgba.png").unwrap();
    println!("Loaded RGBA image with size {:?}", img.dimensions());

    // Convert the image to a blit buffer
    let rgba = img.into_rgba8().to_blit_buffer_with_alpha(127);

    // Load another image with a mask color from disk
    let img = image::open("examples/smiley_rgb.png").unwrap();
    println!("Loaded RGB image with size {:?}", img.dimensions());

    // Also convert it to a blit buffer
    let rgb = img.into_rgb8().to_blit_buffer_with_mask_color(MASK_COLOR);

    // Setup a winit window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowExtWebSys;

        web_sys::window()
            .unwrap()
            .document()
            .body()
            .unwrap()
            .append_child(&window.canvas())
            .unwrap();
    }
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
                    // Clear the buffer first
                    buffer.fill(BACKGROUND_COLOR);

                    // Resize the buffer with empty pictures
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
            } => {
                if window_id == window.id() {
                    // Draw the picture at the center of the cursor
                    let x_pos = mouse_pos.0 - (rgb.width() / 2) as i32;
                    let y_pos = mouse_pos.1 - (rgb.height() / 2) as i32;
                    rgb.blit(&mut buffer, width, (x_pos, y_pos));
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                window_id,
                ..
            } => {
                if window_id == window.id() {
                    mouse_pos.0 = position.x as i32;
                    mouse_pos.1 = position.y as i32;
                }
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
