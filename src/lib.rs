#![crate_name = "blit"]

//! Draw sprites quickly using bitwise operations and a masking color.
//!
//! # Usage
//!
//! This crate is [on crates.io](htpps://crates.io/crates/blit) and can be used by adding
//! `blit` to the dependencies in your project's `Cargo.toml`.
//!
//! ```toml
//! [dependencies]
//! blit = "0.3"
//! ```
//!
//! and this to your crate root:
//!
//! ```rust
//! extern crate blit;
//! ```
//!
//! # Examples
//!
//! ```ignore
//! extern crate image;
//!
//! const WIDTH: i32 = 180;
//! const HEIGHT: i32 = 180;
//! const MASK_COLOR: u32 = 0xFFFF00FF;
//!
//! let mut buffer: Vec<u32> = vec![0xFFFFFFFF; (WIDTH * HEIGHT) as usize];
//!
//! let img = image::open("examples/smiley.png").unwrap();
//! let img_rgb = img.as_rgb8().unwrap();
//!
//! // Blit directly to the buffer
//! let pos = (0, 0);
//! img_rgb.blit_with_mask_color(&mut buffer, (WIDTH, HEIGHT), pos, MASK_COLOR);
//!
//! // Blit by creating a special blitting buffer first, this has some initial
//! // overhead but is a lot faster after multiple calls
//! let blit_buffer = img_rgb.as_blit_buffer(MASK_COLOR);
//!
//! let pos = (10, 10);
//! blit_buffer.blit(&mut buffer, (WIDTH, HEIGHT), pos);
//! let pos = (20, 20);
//! blit_buffer.blit(&mut buffer, (WIDTH, HEIGHT), pos);
//!
//! // Save the blit buffer to a file
//! blit_buffer.save("smiley.blit");
//! ```

extern crate image;
extern crate bincode;
#[macro_use]
extern crate serde_derive;

use std::cmp;
use std::io::{BufWriter, Read};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use image::*;
use bincode::{serialize_into, deserialize, Infinite};

/// A data structure holding a color and a mask buffer to make blitting on a buffer real fast.
#[derive(Serialize, Deserialize)]
pub struct BlitBuffer {
    width: i32,
    height: i32,

    color: Vec<u32>,
    mask: Vec<u32>
}

impl BlitBuffer {
    /// Blit the image on a buffer using bitwise operations--this is a lot faster than
    /// `blit_with_mask_color`.
    pub fn blit(&self, dst: &mut Vec<u32>, dst_size: (i32, i32), offset: (i32, i32)) {
        if offset == (0, 0) && dst_size.0 == self.width && dst_size.1 == self.height {
            // If the sizes match and the buffers are aligned we don't have to do any special
            // bounds checks
            for (pixel, (color, mask)) in dst.iter_mut().zip(self.color.iter().zip(&self.mask)) {
                *pixel = *pixel & *mask | *color;
            }

            return;
        }

        let src_size = (self.width, self.height);

        let dst_start = (cmp::max(offset.0, 0),
                         cmp::max(offset.1, 0));
        let dst_end = (cmp::min(offset.0 + src_size.0, dst_size.0),
                       cmp::min(offset.1 + src_size.1, dst_size.1));

        for dst_y in dst_start.1..dst_end.1 {
            let src_y = dst_y - offset.1;
            
            let dst_y_index = dst_y * dst_size.0;
            let src_y_index = src_y * src_size.0;

            for dst_x in dst_start.0..dst_end.0 {
                let src_x = dst_x - offset.0;

                let src_index = (src_x + src_y_index) as usize;
                let dst_index = (dst_x + dst_y_index) as usize;

                let dst_pixel = &mut dst[dst_index];
                *dst_pixel = *dst_pixel & self.mask[src_index] | self.color[src_index];
            }
        }
    }

    /// Blit a section of the image on a buffer.
    pub fn blit_rect(&self, dst: &mut Vec<u32>, dst_size: (i32, i32), offset: (i32, i32), sub_rect: (i32, i32, i32, i32)) {
        let src_size = (self.width, self.height);

        let dst_start = (cmp::max(offset.0, 0),
                         cmp::max(offset.1, 0));
        let dst_end = (cmp::min(offset.0 + sub_rect.2, dst_size.0),
                       cmp::min(offset.1 + sub_rect.3, dst_size.1));

        for dst_y in dst_start.1..dst_end.1 {
            let src_y = dst_y - offset.1 + sub_rect.1;
            
            let dst_y_index = dst_y * dst_size.0;
            let src_y_index = src_y * src_size.0;

            for dst_x in dst_start.0..dst_end.0 {
                let src_x = dst_x - offset.0 + sub_rect.0;

                let src_index = (src_x + src_y_index) as usize;
                let dst_index = (dst_x + dst_y_index) as usize;

                let dst_pixel = &mut dst[dst_index];

                // First draw the mask as black on the background using an AND operation, and then
                // draw the colors using an OR operation
                *dst_pixel = *dst_pixel & self.mask[src_index] | self.color[src_index];
            }
        }
    }

    // Create a instance from a buffer of `u32` data.
    pub fn from_u32(src: &[u32], width: i32, mask_color: u32) -> Self {
        // TODO optimize

        let height = src.len() as i32 / width;

        let pixels = (width * height) as usize;
        let mut color: Vec<u32> = vec![0; pixels];
        let mut mask: Vec<u32> = vec![0; pixels];

        // Add 0xFF to the beginning of the mask so we can use that in the equality check
        let mask_correct = mask_color | 0xFF000000;

        for index in 0..src.len() {
            let pixel = src[index];

            // Convert pixel to u32
            let raw = 0xFF000000 | pixel;

            if raw == mask_correct {
                mask[index] = 0xFFFFFF;
            } else {
                color[index] = raw;
            }
        }

        BlitBuffer { 
            width: width as i32,
            height: height as i32,
            color,
            mask
        }
    }

    /// Saves the buffer to a file at the path specified.
    /// A custom binary format is used for this.
    pub fn save<P>(&self, path: P) -> Result<(), Box<Error>> where P: AsRef<Path> {
        let mut file = File::create(path)?;
        {
            let mut writer = BufWriter::new(&mut file);
            serialize_into(&mut writer, &self, Infinite)?;
        }
        file.sync_all()?;

        Ok(())
    }

    /// Create a new buffer from a file at the path specified.
    /// The file needs to be the custom binary format.
    pub fn open<P>(path: P) -> Result<Self, Box<Error>> where P: AsRef<Path> {
        let mut file = File::open(path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        BlitBuffer::load_from_memory(&data[..])
    }

    /// Create a new buffer from a file at the path specified.
    /// The array needs to be the custom binary format.
    pub fn load_from_memory(buffer: &[u8]) -> Result<Self, Box<Error>> {
        let buffer = deserialize(buffer)?;

        Ok(buffer)
    }

    /// Get the size of the buffer in pixels.
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }
}

/// A trait adding blitting functions to image types.
pub trait BlitExt {
    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the
    /// blitting operations.
    fn as_blit_buffer(&self, mask_color: u32) -> BlitBuffer;

    /// Blit the image directly on a buffer.
    fn blit_with_mask_color(&self, dst: &mut Vec<u32>, dst_size: (i32, i32), offset: (i32, i32), mask_color: u32);
}

impl BlitExt for RgbImage {
    fn as_blit_buffer(&self, mask_color: u32) -> BlitBuffer {
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
            width: width as i32,
            height: height as i32,
            color,
            mask
        }
    }

    fn blit_with_mask_color(&self, dst: &mut Vec<u32>, dst_size: (i32, i32), offset: (i32, i32), mask_color: u32) {
        let (width, height) = self.dimensions();

        // Add 0xFF to the beginning of the mask so we can use that in the equality check
        let mask_correct = mask_color | 0xFF000000;

        // Make sure only the pixels get rendered that are inside the dst
        let min_x = cmp::max(-offset.0, 0);
        let min_y = cmp::max(-offset.1, 0);

        let max_x = cmp::min(dst_size.0 - offset.0, width as i32);
        let max_y = cmp::min(dst_size.1 - offset.1, height as i32);

        for y in min_y..max_y {
            for x in min_x..max_x {
                let pixel = self.get_pixel(x as u32, y as u32).data;

                // Convert pixel to u32
                let raw = 0xFF000000 | ((pixel[0] as u32) << 16) | ((pixel[1] as u32) << 8) | (pixel[2] as u32);

                // Check if the pixel isn't the mask
                if raw != mask_correct {
                    // Apply the offsets
                    let dst_x = (x + offset.0) as usize;
                    let dst_y = (y + offset.1) as usize;

                    // Calculate the index
                    let index = dst_x + dst_y * dst_size.0 as usize;
                    dst[index] = raw;
                }
            }
        }
    }
}
