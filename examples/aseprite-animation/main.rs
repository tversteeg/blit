use aseprite::SpritesheetData;
use blit::*;
use minifb::*;
use std::fs::File;
use std::time::SystemTime;

const WIDTH: usize = 512;
const HEIGHT: usize = 120;

const MASK_COLOR: u32 = 0xFF_00_FF;

/// Load an aseprite generated spritesheet.
fn load_aseprite_image(partial_path: &str) -> (BlitBuffer, SpritesheetData) {
    let image_path = format!("{partial_path}.png");
    let json_path = format!("{partial_path}.json");

    // Open the animation spritesheet image
    let img = image::open(image_path).unwrap();
    let rgba = img.into_rgb8();
    // Convert it to a blitbuffer
    let blit_buf = rgba.to_blit_buffer_with_mask_color(MASK_COLOR);

    // Open the spritesheet info
    let file = File::open(json_path).unwrap();
    let info: SpritesheetData = serde_json::from_reader(file).unwrap();

    (blit_buf, info)
}

fn main() {
    let mut buffer: Vec<u32> = vec![0x00_FF_FF_FF; WIDTH * HEIGHT];

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new(
        "Blit Animation Example - ESC to exit",
        WIDTH,
        HEIGHT,
        options,
    )
    .expect("Unable to open window");

    // Create the animation buffer object
    let (anim_blit_buf, anim_info) = load_aseprite_image("examples/king-by-buch");
    let anim_buffer = AnimationBlitBuffer::new(anim_blit_buf, anim_info);

    // Create the animations using hardcoded frames
    let mut walk_anim = Animation::start(0, 3, true);
    let mut jump_anim = Animation::start(4, 7, true);
    let mut full_anim = Animation::start(0, 11, true);

    // Create the animation using a tag name
    let mut run_anim = Animation::start_from_tag(&anim_buffer, "Walk".to_string(), true).unwrap();

    let mut time = SystemTime::now();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0x00_FF_FF_FF);

        // Update the animations to go to the correct frame
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
            .blit(&mut buffer, WIDTH, (4, 4), &walk_anim)
            .unwrap();
        anim_buffer
            .blit(&mut buffer, WIDTH, (36, 4), &jump_anim)
            .unwrap();
        anim_buffer
            .blit(&mut buffer, WIDTH, (68, 4), &run_anim)
            .unwrap();
        anim_buffer
            .blit(&mut buffer, WIDTH, (4, 68), &full_anim)
            .unwrap();

        // Draw all the frames separately
        for i in 0..11 {
            anim_buffer
                .blit_frame(&mut buffer, WIDTH, (32 * i + 4, 36), i as usize)
                .unwrap();
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();

        time = SystemTime::now();
    }
}
