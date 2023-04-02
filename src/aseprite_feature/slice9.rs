use std::error::Error;

use aseprite::SpritesheetData;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::BlitBuffer;

/// `BlitBuffer` with extra information for rendering as a scalable slice 9 graphic.
///
/// The slice information for scaling it should be exported in the output JSON from aseprite.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
pub struct AnimationBlitBuffer {
    buffer: BlitBuffer,
    info: SpritesheetData,
}

impl AnimationBlitBuffer {
    /// Construct a new buffer for animating a spritesheet.
    pub fn new(buffer: BlitBuffer, info: SpritesheetData) -> Self {
        AnimationBlitBuffer { buffer, info }
    }

    /// Draw the current frame using the animation info.
    pub fn blit(
        &self,
        dst: &mut [u32],
        dst_width: usize,
        offset: (i32, i32),
    ) -> Result<(), Box<dyn Error>> {
        todo!()
    }
}
