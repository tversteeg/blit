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
//! blit = "0.4"
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
//! const WIDTH: usize = 180;
//! const HEIGHT: usize = 180;
//! const MASK_COLOR: u32 = 0xFF00FF;
//!
//! let mut buffer: Vec<u32> = vec![0xFFFFFFFF; WIDTH * HEIGHT];
//!
//! let img = image::open("examples/smiley.png").unwrap();
//! let img_rgb = img.as_rgb8().unwrap();
//!
//! // Blit directly to the buffer
//! let pos = (0, 0);
//! img_rgb.blit(&mut buffer, WIDTH, pos, Color::from_u32(MASK_COLOR));
//!
//! // Blit by creating a special blitting buffer first, this has some initial
//! // overhead but is a lot faster after multiple calls
//! let blit_buffer = img_rgb.to_blit_buffer(Color::from_u32(MASK_COLOR));
//!
//! let pos = (10, 10);
//! blit_buffer.blit(&mut buffer, WIDTH, pos);
//! let pos = (20, 20);
//! blit_buffer.blit(&mut buffer, WIDTH, pos);
//!
//! // Save the blit buffer to a file
//! blit_buffer.save("smiley.blit");
//! ```

extern crate bincode;
#[macro_use] extern crate serde_derive;
#[cfg(feature = "image")] extern crate image;
#[cfg(feature = "aseprite")] extern crate aseprite;

use std::cmp;
use std::io::{BufWriter, Read};
use std::fs::File;
use std::path::Path;
use std::error::Error;
use bincode::{serialize_into, deserialize};

#[cfg(feature = "image")]#[doc(hidden)] pub mod image_feature;
#[cfg(feature = "aseprite")] pub mod aseprite_feature;
#[cfg(feature = "aseprite")] pub use aseprite_feature::AnimationBlitBuffer;

/// A trait so that both `Color` and `u32` can do blitting operations.
trait BlittablePrimitive {
    /// First draw the mask as black on the background using an AND operation, and then draw the color using an OR operation.
    fn blit(&mut self, color: Self, mask: Self);
}

/// A newtype representing the color in a buffer.
///
/// It is divided into alpha (not used), red, green & blue:
/// 0xFF000000: alpha (not used)
/// 0x00FF0000: red
/// 0x0000FF00: green
/// 0x000000FF: blue
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Color(u32);

impl Color {
    /// Create a color from a 32 bits unsigned int, discard the alpha (last 8 bits).
    pub fn from_u32(color: u32) -> Self {
        Color(color | 0xFF000000)
    }

    /// Create a color from 3 8 bits unsigned ints and pack them into a 32 bit unsigned int.
    pub fn from_u8(red: u8, green: u8, blue: u8) -> Self {
        Color(0xFF000000 | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue))
    }

    /// Return the wrapped `u32` object.
    pub fn u32(&self) -> u32 {
        self.0
    }
}

impl BlittablePrimitive for Color {
    fn blit(&mut self, color: Self, mask: Self) {
        self.0 = self.0 & mask.0 | color.0;
    }
}

impl BlittablePrimitive for u32 {
    fn blit(&mut self, color: Self, mask: Self) {
        *self = *self & mask | color;
    }
}

/// A data structure holding a color and a mask buffer to make blitting on a buffer real fast.
#[derive(Serialize, Deserialize, Debug)]
pub struct BlitBuffer {
    width: i32,
    height: i32,

    color: Vec<Color>,
    mask: Vec<Color>
}

impl BlitBuffer {
    /// Blit the image on a buffer using bitwise operations--this is a lot faster than
    /// `blit_with_mask_color`.
    pub fn blit(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32)) {
        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        if offset == (0, 0) && dst_size.0 == self.width && dst_size.1 == self.height {
            // If the sizes match and the buffers are aligned we don't have to do any special
            // bounds checks
            for (pixel, (color, mask)) in dst.iter_mut().zip(self.color.iter().zip(&self.mask)) {
                pixel.blit(color.u32(), mask.u32());
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

                // First draw the mask as black on the background using an AND operation, and then
                // draw the colors using an OR operation
                dst[dst_index].blit(self.color[src_index].u32(), self.mask[src_index].u32());
            }
        }
    }

    /// Blit a section of the image on a buffer.
    pub fn blit_rect(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32), sub_rect: (i32, i32, i32, i32)) {
        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

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

                // First draw the mask as black on the background using an AND operation, and then
                // draw the colors using an OR operation
                dst[dst_index].blit(self.color[src_index].u32(), self.mask[src_index].u32());
            }
        }
    }

    /// Create a instance from a buffer of `Color` data.
    pub fn from_buffer(src: &[u32], width: i32, mask_color: Color) -> Self {
        let height = src.len() as i32 / width;

        let pixels = (width * height) as usize;
        let mut color: Vec<Color> = vec![Color::from_u32(0); pixels];
        let mut mask: Vec<Color> = vec![Color::from_u32(0); pixels];

        for index in 0..src.len() {
            let pixel = Color::from_u32(src[index]);

            if pixel == mask_color {
                mask[index] = Color::from_u32(0xFFFFFF);
            } else {
                color[index] = pixel;
            }
        }

        BlitBuffer { width, height, color, mask }
    }

    /// Saves the buffer to a file at the path specified.
    /// A custom binary format is used for this.
    pub fn save<P>(&self, path: P) -> Result<(), Box<Error>> where P: AsRef<Path> {
        let mut file = File::create(path)?;
        {
            let mut writer = BufWriter::new(&mut file);
            serialize_into(&mut writer, &self)?;
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

        BlitBuffer::from_memory(&data[..])
    }

    /// Create a new buffer from a file at the path specified.
    /// The array needs to be the custom binary format.
    pub fn from_memory(buffer: &[u8]) -> Result<Self, Box<Error>> {
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
    fn to_blit_buffer(&self, mask_color: Color) -> BlitBuffer;

    /// Blit the image directly on a buffer.
    fn blit(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32), mask_color: Color);
}
