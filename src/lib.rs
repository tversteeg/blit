extern crate image;

use image::*;

/// A trait adding blitting functions to image types.
pub trait BlitExt {
    /// Blit the image on a buffer.
    fn blit_on_buffer(&self, buffer: &mut Vec<u32>, buffer_width: usize, pos: (i32, i32), mask_color: u32);
}

impl BlitExt for DynamicImage {
    fn blit_on_buffer(&self, buffer: &mut Vec<u32>, buffer_width: usize, pos: (i32, i32), mask_color: u32) {
        buffer[0] = 0x00FFFFFF;
    }
}
