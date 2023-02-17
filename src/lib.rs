#![crate_name = "blit"]

//! Draw sprites quickly using bitwise operations and a masking color.
//!
//! # Example
//!
//! ```
//! use blit::{BlitExt, Color};
//!
//! const WIDTH: usize = 180;
//! const HEIGHT: usize = 180;
//! const MASK_COLOR: u32 = 0xFF_00_FF;
//!
//! let mut buffer: Vec<u32> = vec![0xFF_FF_FF_FF; WIDTH * HEIGHT];
//!
//! let img = image::open("examples/smiley_rgb.png").unwrap();
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
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate quick_error;
#[cfg(feature = "aseprite")]
extern crate aseprite;
#[cfg(feature = "image")]
extern crate image;
#[cfg(feature = "image")]
extern crate num_traits;

use bincode::{deserialize, serialize_into};
use std::{
    cmp,
    error::Error,
    fmt,
    fs::File,
    io::{BufWriter, Read},
    path::Path,
};

#[cfg(feature = "image")]
pub mod image_feature;
#[cfg(feature = "image")]
pub use image_feature::*;
#[cfg(feature = "aseprite")]
pub mod aseprite_feature;
#[cfg(feature = "aseprite")]
pub use aseprite_feature::*;

/// A trait so that both `Color` and `u32` can do blitting operations.
trait BlittablePrimitive {
    /// First draw the mask as black on the background using an AND operation, and then draw the color using an OR operation.
    fn blit(&mut self, color: Self, mask: Self);
}

/// A newtype representing the color in a buffer.
///
/// It is divided into alpha (not used), red, green & blue:
/// 0xFF_00_00_00: alpha (not used)
/// 0x00_FF_00_00: red
/// 0x00_00_FF_00: green
/// 0x00_00_00_FF: blue
#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Default)]
pub struct Color(u32);

impl Color {
    /// Create a color from a 32 bits unsigned int, discard the alpha (last 8 bits).
    pub fn from_u32(color: u32) -> Self {
        Color(color | 0xFF_00_00_00)
    }

    /// Create a color from 3 8 bits unsigned ints and pack them into a 32 bit unsigned int.
    pub fn from_u8(red: u8, green: u8, blue: u8) -> Self {
        Color(0xFF_00_00_00 | (u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue))
    }

    /// Return the wrapped `u32` object.
    pub fn u32(self) -> u32 {
        self.0
    }

    /// Alpha color.
    ///
    /// This is a special color and won't be rendered on top of the other colors.
    pub fn alpha() -> Self {
        Self(0xFF_00_00_00)
    }

    /// White color.
    pub fn white() -> Self {
        Self(0x00_FF_FF_FF)
    }

    /// Black color.
    pub fn black() -> Self {
        Self(0x00_00_00_00)
    }
}

impl BlittablePrimitive for Color {
    fn blit(&mut self, color: Self, mask: Self) {
        self.0.blit(mask.0, color.0);
    }
}

impl From<u32> for Color {
    fn from(raw: u32) -> Self {
        Self::from_u32(raw)
    }
}

impl fmt::Debug for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{:x?}", self.0))
    }
}

impl BlittablePrimitive for u32 {
    fn blit(&mut self, color: Self, mask: Self) {
        // First draw the mask as black on the background using an AND operation, and then draw the colors using an OR operation
        *self = *self & mask | color;
    }
}

/// A data structure holding a color and a mask buffer to make blitting on a buffer real fast.
#[derive(Serialize, Deserialize, Debug)]
pub struct BlitBuffer {
    width: i32,
    height: i32,

    // The first field of the tuple is the color, the second the mask
    data: Vec<(Color, Color)>,

    mask_color: Color,
}

impl BlitBuffer {
    /// Blit the image on a buffer using bitwise operations--this is a lot faster than
    /// `blit_with_mask_color`.
    pub fn blit(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32)) {
        let src_size = (self.width, self.height);
        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        if offset == (0, 0) && dst_size == src_size {
            // If the sizes match and the buffers are aligned we don't have to do any special
            // bounds checks
            dst.iter_mut()
                .zip(&self.data[..])
                .for_each(|(pixel, color)| {
                    pixel.blit(color.0.u32(), color.1.u32());
                });

            return;
        }

        let dst_start = (cmp::max(offset.0, 0), cmp::max(offset.1, 0));
        let dst_end = (
            cmp::min(offset.0 + src_size.0, dst_size.0),
            cmp::min(offset.1 + src_size.1, dst_size.1),
        );

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
                let (color, mask) = self.data[src_index];
                dst[dst_index].blit(color.u32(), mask.u32());
            }
        }
    }

    /// Blit a section of the image on a buffer.
    pub fn blit_rect(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        offset: (i32, i32),
        sub_rect: (i32, i32, i32, i32),
    ) {
        let dst_size = (dst_width as i32, (dst.len() / dst_width) as i32);

        let src_size = (self.width, self.height);

        let dst_start = (cmp::max(offset.0, 0), cmp::max(offset.1, 0));
        let dst_end = (
            cmp::min(offset.0 + sub_rect.2, dst_size.0),
            cmp::min(offset.1 + sub_rect.3, dst_size.1),
        );

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
                let (color, mask) = self.data[src_index];
                dst[dst_index].blit(color.u32(), mask.u32());
            }
        }
    }

    /// Create a instance from a buffer of `Color` data.
    pub fn from_buffer<C>(src: &[u32], width: i32, mask_color: C) -> Self
    where
        C: Into<Color>,
    {
        let mask_color = mask_color.into();

        let height = src.len() as i32 / width;

        let pixels = (width * height) as usize;
        let mut data: Vec<(Color, Color)> = vec![(Color::from_u32(0), Color::from_u32(0)); pixels];

        for index in 0..src.len() {
            let pixel = Color::from_u32(src[index]);

            if pixel == mask_color {
                // Set the mask
                data[index].1 = Color::from_u32(0xFF_FF_FF);
            } else {
                // Set the color
                data[index].0 = pixel;
            }
        }

        BlitBuffer {
            width,
            height,
            data,
            mask_color,
        }
    }

    /// Saves the buffer to a file at the path specified.
    /// A custom binary format is used for this.
    pub fn save<P>(&self, path: P) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
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
    pub fn open<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let mut file = File::open(path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        BlitBuffer::from_memory(&data[..])
    }

    /// Create a new buffer from a file at the path specified.
    /// The array needs to be the custom binary format.
    pub fn from_memory(buffer: &[u8]) -> Result<Self, Box<dyn Error>> {
        let buffer = deserialize(buffer)?;

        Ok(buffer)
    }

    /// Get the size of the buffer in pixels.
    pub fn size(&self) -> (i32, i32) {
        (self.width, self.height)
    }

    /// Get the width of the buffer in pixels.
    pub fn width(&self) -> i32 {
        self.width
    }

    /// Get the height of the buffer in pixels.
    pub fn height(&self) -> i32 {
        self.height
    }

    /// Get the color of the mask.
    pub fn mask_color(&self) -> Color {
        self.mask_color
    }

    /// Get the raw pixels buffer, this is a costly operation.
    pub fn to_raw_buffer(&self) -> Vec<u32> {
        self.data
            .iter()
            .map(|(color, _mask)| color.0)
            .collect::<_>()
    }
}

/// A trait adding blitting functions to image types.
pub trait BlitExt {
    /// Convert the image to a custom `BlitBuffer` type which is optimized for applying the
    /// blitting operations.
    fn to_blit_buffer<C>(&self, mask_color: C) -> BlitBuffer
    where
        C: Into<Color>;

    /// Blit the image directly on a buffer.
    fn blit<C>(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32), mask_color: C)
    where
        C: Into<Color>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_fit() {
        let mut buffer = [0xFF, 0xFF_00, 0xFF_00_00, 0xFF, 0xFF_00, 0xFF_00_00];

        // The last number should be masked
        let blit = BlitBuffer::from_buffer(&[0xAA, 0xAA_00, 0xAA_00_00, 0xBB, 0xBB, 0xBB], 2, 0xBB);
        blit.blit(&mut buffer, 2, (0, 0));

        // Create a copy but cast the u32 to a i32
        assert_eq!(
            buffer,
            [
                0xAA | 0xFF_00_00_00,
                0xAA_00 | 0xFF_00_00_00,
                0xAA_00_00 | 0xFF_00_00_00,
                0xFF | 0xFF_00_00_00,
                0xFF_00 | 0xFF_00_00_00,
                0xFF_00_00 | 0xFF_00_00_00,
            ]
        );
    }
}
