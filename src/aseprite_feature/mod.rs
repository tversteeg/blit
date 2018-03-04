use super::*;
use aseprite::*;

/// `BlitBuffer` with extra information and functions to animate a sheet.
#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationBlitBuffer {
    buffer: BlitBuffer,
    info: SpritesheetData
}

impl AnimationBlitBuffer {
    pub fn new(buffer: BlitBuffer, info: SpritesheetData) -> Self {
        AnimationBlitBuffer {
            buffer, info
        }
    }

    pub fn blit_frame(&self, dst: &mut [u32], dst_width: usize, offset: (i32, i32), frame: usize) -> Result<(), Box<Error>> {
        let frame = &self.info.frames[frame];

        let rect = (frame.frame.x as i32,
                    frame.frame.y as i32,
                    frame.frame.w as i32,
                    frame.frame.h as i32);
        self.buffer.blit_rect(dst, dst_width, offset, rect);

        Ok(())
    }
}
