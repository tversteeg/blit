extern crate image;

use image::*;
use std::cmp;

pub struct BlitBuffer {
    width: usize,
    height: usize,

    color: Vec<u32>,
    mask: Vec<u32>
}

impl BlitBuffer {
    /// Blit the image on a buffer using bitwise operations--this is a lot faster than
    /// `blit_with_mask_color`.
    pub fn blit(&self, buffer: &mut Vec<u32>, buffer_size: (usize, usize), pos: (i32, i32)) {
        // Make sure only the pixels get rendered that are inside the buffer
        let min_x = cmp::max(-pos.0, 0);
        let min_y = cmp::max(-pos.1, 0);

        let max_x = cmp::min(buffer_size.0 as i32 - pos.0, self.width as i32);
        let max_y = cmp::min(buffer_size.1 as i32 - pos.1, self.height as i32);

        for y in min_y..max_y {
            let y_index = y as usize * self.width;

            // Apply the offsets
            let buffer_y = (y + pos.1) as usize;
            for x in min_x..max_x {
                // Apply the offsets
                let buffer_x = (x + pos.0) as usize;

                // Calculate the index of the buffer
                let buffer_index = buffer_x + buffer_y * buffer_size.0;
                let mut pixel = buffer[buffer_index];

                // Calculate the index of the source image
                let index = x as usize + y_index;

                // First draw the mask as black on the background using an AND operation, and then
                // draw the colors using an OR operation
                pixel = pixel & self.mask[index] | self.color[index];

                buffer[buffer_index] = pixel;
            }
        }
    }
}

/// A trait adding blitting functions to image types.
pub trait BlitExt {
    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the
    /// blitting operations.
    fn to_buffer(&self, mask_color: u32) -> BlitBuffer;

    /// Blit the image directly on a buffer.
    fn blit_with_mask_color(&self, buffer: &mut Vec<u32>, buffer_size: (usize, usize), pos: (i32, i32), mask_color: u32);
}

impl BlitExt for RgbImage {
    fn to_buffer(&self, mask_color: u32) -> BlitBuffer {
        let (width, height) = self.dimensions();

        let pixels = (width * height) as usize;
        let mut color: Vec<u32> = vec![0; pixels];
        let mut mask: Vec<u32> = vec![0; pixels];

        // Add 0xFF to the beginning of the mask so we can use that in the equality check
        let mask_correct = mask_color | 0xFF000000;

        let mut index = 0;
        for y in 0..height {
            for x in 0..width {
                let pixel = self.get_pixel(x, y).data;

                // Convert pixel to u32
                let raw = 0xFF000000 | ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);

                if raw == mask_correct {
                    mask[index] = 0xFFFFFF;
                } else {
                    color[index] = raw;
                }

                index += 1;
            }
        }

        BlitBuffer {
            width: width as usize,
            height: height as usize,
            color,
            mask
        }
    }

    fn blit_with_mask_color(&self, buffer: &mut Vec<u32>, buffer_size: (usize, usize), pos: (i32, i32), mask_color: u32) {
        let (width, height) = self.dimensions();

        // Add 0xFF to the beginning of the mask so we can use that in the equality check
        let mask_correct = mask_color | 0xFF000000;

        // Make sure only the pixels get rendered that are inside the buffer
        let min_x = cmp::max(-pos.0, 0);
        let min_y = cmp::max(-pos.1, 0);

        let max_x = cmp::min(buffer_size.0 as i32 - pos.0, width as i32);
        let max_y = cmp::min(buffer_size.1 as i32 - pos.1, height as i32);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let pixel = self.get_pixel(x as u32, y as u32).data;

                // Convert pixel to u32
                let raw = 0xFF000000 | ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);

                // Check if the pixel isn't the mask
                if raw != mask_correct {
                    // Apply the offsets
                    let buffer_x = (x + pos.0) as usize;
                    let buffer_y = (y + pos.1) as usize;

                    // Calculate the index
                    let index = buffer_x + buffer_y * buffer_size.0;
                    buffer[index] = raw;
                }
            }
        }
    }
}
