extern crate image;

use image::*;
use std::cmp;

/// A trait adding blitting functions to image types.
pub trait BlitExt {
    /// Blit the image on a buffer.
    fn blit_with_mask_color(&self, buffer: &mut Vec<u32>, buffer_size: (usize, usize), pos: (i32, i32), mask_color: u32);
}

impl BlitExt for RgbImage {
    fn blit_with_mask_color(&self, buffer: &mut Vec<u32>, buffer_size: (usize, usize), pos: (i32, i32), mask_color: u32) {
        let (width, height) = self.dimensions();

        // Add FF to the beginning of the mask so we can use that in the equality check
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
